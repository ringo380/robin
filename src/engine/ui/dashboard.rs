use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::time::{Instant, Duration};
use nalgebra::{Vector2, Vector3, Vector4};
use crate::engine::error::RobinResult;
use crate::engine::ui::production_ui::{
    UIComponent, ComponentType, GraphComponent, DataSeries, GraphType,
    TextComponent, PanelComponent, SliderComponent, Rect, Color, TextAlignment,
};

/// Real-time Performance Dashboard for Production UI
#[derive(Debug)]
pub struct PerformanceDashboard {
    pub metrics_collector: MetricsCollector,
    pub fps_monitor: FPSMonitor,
    pub memory_monitor: MemoryMonitor,
    pub gpu_monitor: GPUMonitor,
    pub cpu_monitor: CPUMonitor,
    pub optimization_visualizer: OptimizationVisualizer,
    pub statistics_panel: StatisticsPanel,
    pub alert_system: AlertSystem,
    pub layout_manager: DashboardLayout,
    config: DashboardConfig,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub position: DashboardPosition,
    pub size: DashboardSize,
    pub opacity: f32,
    pub update_frequency_hz: u32,
    pub graph_history_seconds: f32,
    pub show_fps: bool,
    pub show_memory: bool,
    pub show_gpu: bool,
    pub show_cpu: bool,
    pub show_optimization_metrics: bool,
    pub show_alerts: bool,
    pub compact_mode: bool,
    pub auto_hide: bool,
    pub auto_hide_delay_seconds: f32,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            position: DashboardPosition::TopRight,
            size: DashboardSize::Normal,
            opacity: 0.9,
            update_frequency_hz: 60,
            graph_history_seconds: 10.0,
            show_fps: true,
            show_memory: true,
            show_gpu: true,
            show_cpu: true,
            show_optimization_metrics: true,
            show_alerts: true,
            compact_mode: false,
            auto_hide: false,
            auto_hide_delay_seconds: 5.0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DashboardPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Custom(f32, f32),
}

#[derive(Debug, Clone, Copy)]
pub enum DashboardSize {
    Compact,
    Normal,
    Extended,
    Custom(f32, f32),
}

impl PerformanceDashboard {
    pub fn new(config: DashboardConfig) -> RobinResult<Self> {
        let metrics_collector = MetricsCollector::new(config.update_frequency_hz);
        let fps_monitor = FPSMonitor::new(config.graph_history_seconds);
        let memory_monitor = MemoryMonitor::new(config.graph_history_seconds);
        let gpu_monitor = GPUMonitor::new(config.graph_history_seconds);
        let cpu_monitor = CPUMonitor::new(config.graph_history_seconds);
        let optimization_visualizer = OptimizationVisualizer::new();
        let statistics_panel = StatisticsPanel::new();
        let alert_system = AlertSystem::new();
        let layout_manager = DashboardLayout::new(&config);

        Ok(Self {
            metrics_collector,
            fps_monitor,
            memory_monitor,
            gpu_monitor,
            cpu_monitor,
            optimization_visualizer,
            statistics_panel,
            alert_system,
            layout_manager,
            config,
            enabled: true,
        })
    }

    pub fn update(&mut self, delta_time: Duration) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Collect current metrics
        let metrics = self.metrics_collector.collect()?;

        // Update monitors
        if self.config.show_fps {
            self.fps_monitor.update(&metrics)?;
        }

        if self.config.show_memory {
            self.memory_monitor.update(&metrics)?;
        }

        if self.config.show_gpu {
            self.gpu_monitor.update(&metrics)?;
        }

        if self.config.show_cpu {
            self.cpu_monitor.update(&metrics)?;
        }

        // Update optimization metrics
        if self.config.show_optimization_metrics {
            self.optimization_visualizer.update(&metrics)?;
        }

        // Update statistics
        self.statistics_panel.update(&metrics)?;

        // Check for alerts
        if self.config.show_alerts {
            self.alert_system.check_alerts(&metrics)?;
        }

        Ok(())
    }

    pub fn create_ui_components(&self) -> Vec<UIComponent> {
        let mut components = Vec::new();
        let layout = &self.layout_manager;

        // Main dashboard panel
        let panel_bounds = layout.get_panel_bounds();
        let main_panel = UIComponent::new_panel(
            "dashboard_panel",
            panel_bounds,
            PanelComponent {
                glass_effect: true,
                blur_intensity: 8.0,
                cast_shadow: true,
                show_border: true,
            },
        );
        components.push(main_panel);

        // Dashboard title
        let title = UIComponent::new_text(
            "dashboard_title",
            layout.get_title_bounds(),
            TextComponent {
                content: "Performance Dashboard".to_string(),
                font_size: 16.0,
                color: Some(Color::white()),
                alignment: TextAlignment::Center,
                drop_shadow: true,
            },
        );
        components.push(title);

        // FPS Graph
        if self.config.show_fps {
            let fps_graph = self.create_fps_graph_component(layout);
            components.push(fps_graph);
        }

        // Memory Graph
        if self.config.show_memory {
            let memory_graph = self.create_memory_graph_component(layout);
            components.push(memory_graph);
        }

        // GPU Usage
        if self.config.show_gpu {
            let gpu_component = self.create_gpu_component(layout);
            components.push(gpu_component);
        }

        // CPU Usage
        if self.config.show_cpu {
            let cpu_component = self.create_cpu_component(layout);
            components.push(cpu_component);
        }

        // Optimization Metrics
        if self.config.show_optimization_metrics {
            let opt_components = self.create_optimization_components(layout);
            components.extend(opt_components);
        }

        // Statistics Panel
        if !self.config.compact_mode {
            let stats_components = self.create_statistics_components(layout);
            components.extend(stats_components);
        }

        // Alert notifications
        if self.config.show_alerts {
            let alert_components = self.alert_system.get_active_alert_components();
            components.extend(alert_components);
        }

        components
    }

    fn create_fps_graph_component(&self, layout: &DashboardLayout) -> UIComponent {
        let bounds = layout.get_fps_graph_bounds();

        UIComponent::new_graph(
            "fps_graph",
            bounds,
            GraphComponent {
                data_series: vec![
                    DataSeries {
                        name: "FPS".to_string(),
                        values: self.fps_monitor.get_history_values(),
                        color: Some(Color::new(0.2, 1.0, 0.3, 1.0)),
                    },
                    DataSeries {
                        name: "Target".to_string(),
                        values: vec![60.0; self.fps_monitor.get_history_size()],
                        color: Some(Color::new(0.8, 0.8, 0.2, 0.5)),
                    },
                ],
                graph_type: GraphType::Line,
                x_axis_label: Some("Time (s)".to_string()),
                y_axis_label: Some("FPS".to_string()),
                show_legend: true,
                show_grid: true,
            },
        )
    }

    fn create_memory_graph_component(&self, layout: &DashboardLayout) -> UIComponent {
        let bounds = layout.get_memory_graph_bounds();

        UIComponent::new_graph(
            "memory_graph",
            bounds,
            GraphComponent {
                data_series: vec![
                    DataSeries {
                        name: "Used".to_string(),
                        values: self.memory_monitor.get_used_memory_history(),
                        color: Some(Color::new(0.8, 0.3, 0.3, 1.0)),
                    },
                    DataSeries {
                        name: "Allocated".to_string(),
                        values: self.memory_monitor.get_allocated_memory_history(),
                        color: Some(Color::new(0.3, 0.5, 0.8, 1.0)),
                    },
                ],
                graph_type: GraphType::Area,
                x_axis_label: Some("Time (s)".to_string()),
                y_axis_label: Some("Memory (MB)".to_string()),
                show_legend: true,
                show_grid: true,
            },
        )
    }

    fn create_gpu_component(&self, layout: &DashboardLayout) -> UIComponent {
        let bounds = layout.get_gpu_bounds();

        // GPU utilization slider showing current usage
        UIComponent::new_slider(
            "gpu_usage",
            bounds,
            SliderComponent {
                value: self.gpu_monitor.get_current_utilization() / 100.0,
                min: 0.0,
                max: 100.0,
                step: 1.0,
                show_value: true,
            },
        )
    }

    fn create_cpu_component(&self, layout: &DashboardLayout) -> UIComponent {
        let bounds = layout.get_cpu_bounds();

        // CPU utilization with per-core breakdown
        UIComponent::new_graph(
            "cpu_usage",
            bounds,
            GraphComponent {
                data_series: self.cpu_monitor.get_per_core_series(),
                graph_type: GraphType::Bar,
                x_axis_label: Some("Core".to_string()),
                y_axis_label: Some("Usage %".to_string()),
                show_legend: false,
                show_grid: true,
            },
        )
    }

    fn create_optimization_components(&self, layout: &DashboardLayout) -> Vec<UIComponent> {
        let mut components = Vec::new();

        // Frustum culling efficiency
        let culling_text = UIComponent::new_text(
            "culling_efficiency",
            layout.get_optimization_text_bounds(0),
            TextComponent {
                content: format!(
                    "Culling: {:.1}% ({}/{} chunks)",
                    self.optimization_visualizer.get_culling_efficiency() * 100.0,
                    self.optimization_visualizer.get_visible_chunks(),
                    self.optimization_visualizer.get_total_chunks()
                ),
                font_size: 12.0,
                color: Some(self.get_efficiency_color(self.optimization_visualizer.get_culling_efficiency())),
                alignment: TextAlignment::Left,
                drop_shadow: false,
            },
        );
        components.push(culling_text);

        // Vertex reduction
        let vertex_text = UIComponent::new_text(
            "vertex_reduction",
            layout.get_optimization_text_bounds(1),
            TextComponent {
                content: format!(
                    "Vertex Reduction: {:.1}% ({} vertices)",
                    self.optimization_visualizer.get_vertex_reduction() * 100.0,
                    self.optimization_visualizer.get_vertex_count()
                ),
                font_size: 12.0,
                color: Some(self.get_efficiency_color(self.optimization_visualizer.get_vertex_reduction())),
                alignment: TextAlignment::Left,
                drop_shadow: false,
            },
        );
        components.push(vertex_text);

        // Draw calls
        let draw_calls_text = UIComponent::new_text(
            "draw_calls",
            layout.get_optimization_text_bounds(2),
            TextComponent {
                content: format!(
                    "Draw Calls: {} (batched: {})",
                    self.optimization_visualizer.get_draw_calls(),
                    self.optimization_visualizer.get_batched_calls()
                ),
                font_size: 12.0,
                color: Some(Color::gray(0.9)),
                alignment: TextAlignment::Left,
                drop_shadow: false,
            },
        );
        components.push(draw_calls_text);

        components
    }

    fn create_statistics_components(&self, layout: &DashboardLayout) -> Vec<UIComponent> {
        let mut components = Vec::new();

        // Frame time breakdown
        let frame_breakdown = UIComponent::new_graph(
            "frame_breakdown",
            layout.get_statistics_bounds(),
            GraphComponent {
                data_series: vec![
                    DataSeries {
                        name: "Update".to_string(),
                        values: vec![self.statistics_panel.get_update_time()],
                        color: Some(Color::new(0.3, 0.8, 0.3, 1.0)),
                    },
                    DataSeries {
                        name: "Render".to_string(),
                        values: vec![self.statistics_panel.get_render_time()],
                        color: Some(Color::new(0.3, 0.3, 0.8, 1.0)),
                    },
                    DataSeries {
                        name: "Present".to_string(),
                        values: vec![self.statistics_panel.get_present_time()],
                        color: Some(Color::new(0.8, 0.3, 0.8, 1.0)),
                    },
                ],
                graph_type: GraphType::Bar,
                x_axis_label: None,
                y_axis_label: Some("ms".to_string()),
                show_legend: true,
                show_grid: false,
            },
        );
        components.push(frame_breakdown);

        components
    }

    fn get_efficiency_color(&self, efficiency: f32) -> Color {
        if efficiency > 0.8 {
            Color::new(0.2, 1.0, 0.3, 1.0) // Green
        } else if efficiency > 0.6 {
            Color::new(1.0, 0.8, 0.2, 1.0) // Yellow
        } else {
            Color::new(1.0, 0.3, 0.3, 1.0) // Red
        }
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn set_position(&mut self, position: DashboardPosition) {
        self.config.position = position;
        self.layout_manager.update_position(position);
    }

    pub fn set_compact_mode(&mut self, compact: bool) {
        self.config.compact_mode = compact;
        self.layout_manager.set_compact_mode(compact);
    }
}

/// Metrics Collector for gathering performance data
#[derive(Debug)]
pub struct MetricsCollector {
    update_frequency: u32,
    last_update: Instant,
}

impl MetricsCollector {
    pub fn new(update_frequency: u32) -> Self {
        Self {
            update_frequency,
            last_update: Instant::now(),
        }
    }

    pub fn collect(&mut self) -> RobinResult<PerformanceMetrics> {
        let now = Instant::now();
        let delta = now.duration_since(self.last_update);
        self.last_update = now;

        Ok(PerformanceMetrics {
            frame_time_ms: delta.as_secs_f32() * 1000.0,
            fps: 1.0 / delta.as_secs_f32(),
            memory_used_mb: Self::get_memory_usage(),
            memory_allocated_mb: Self::get_allocated_memory(),
            gpu_utilization: Self::get_gpu_utilization(),
            gpu_memory_mb: Self::get_gpu_memory(),
            cpu_utilization: Self::get_cpu_utilization(),
            cpu_cores: Self::get_cpu_core_usage(),
            draw_calls: 245,
            vertices_rendered: 125000,
            triangles_rendered: 41666,
            texture_memory_mb: 128.5,
            culling_efficiency: 0.92,
            vertex_reduction: 0.75,
            timestamp: now,
        })
    }

    fn get_memory_usage() -> f32 {
        512.5 // Simulated value
    }

    fn get_allocated_memory() -> f32 {
        768.0 // Simulated value
    }

    fn get_gpu_utilization() -> f32 {
        65.5 // Simulated value
    }

    fn get_gpu_memory() -> f32 {
        1024.0 // Simulated value
    }

    fn get_cpu_utilization() -> f32 {
        42.3 // Simulated value
    }

    fn get_cpu_core_usage() -> Vec<f32> {
        vec![45.2, 38.7, 42.1, 39.8, 44.5, 41.2, 40.9, 43.6] // 8 cores
    }
}

/// FPS Monitor for tracking frame rate over time
#[derive(Debug)]
pub struct FPSMonitor {
    history_duration: f32,
    history: VecDeque<(Instant, f32)>,
}

impl FPSMonitor {
    pub fn new(history_duration: f32) -> Self {
        Self {
            history_duration,
            history: VecDeque::new(),
        }
    }

    pub fn update(&mut self, metrics: &PerformanceMetrics) -> RobinResult<()> {
        self.history.push_back((metrics.timestamp, metrics.fps));

        // Remove old entries
        let cutoff = metrics.timestamp - Duration::from_secs_f32(self.history_duration);
        while let Some((time, _)) = self.history.front() {
            if *time < cutoff {
                self.history.pop_front();
            } else {
                break;
            }
        }

        Ok(())
    }

    pub fn get_history_values(&self) -> Vec<f32> {
        self.history.iter().map(|(_, fps)| *fps).collect()
    }

    pub fn get_history_size(&self) -> usize {
        self.history.len()
    }

    pub fn get_current_fps(&self) -> f32 {
        self.history.back().map(|(_, fps)| *fps).unwrap_or(60.0)
    }

    pub fn get_average_fps(&self) -> f32 {
        if self.history.is_empty() {
            return 60.0;
        }
        let sum: f32 = self.history.iter().map(|(_, fps)| fps).sum();
        sum / self.history.len() as f32
    }
}

// Simplified implementations for other monitors

macro_rules! define_monitor {
    ($name:ident) => {
        #[derive(Debug)]
        pub struct $name {
            history_duration: f32,
            history: VecDeque<(Instant, f32)>,
        }

        impl $name {
            pub fn new(history_duration: f32) -> Self {
                Self {
                    history_duration,
                    history: VecDeque::new(),
                }
            }

            pub fn update(&mut self, _metrics: &PerformanceMetrics) -> RobinResult<()> {
                Ok(())
            }
        }
    };
}

define_monitor!(MemoryMonitor);
define_monitor!(GPUMonitor);
define_monitor!(CPUMonitor);

impl MemoryMonitor {
    pub fn get_used_memory_history(&self) -> Vec<f32> {
        vec![512.0, 515.0, 520.0, 518.0, 522.0, 525.0, 523.0, 527.0, 530.0, 528.0]
    }

    pub fn get_allocated_memory_history(&self) -> Vec<f32> {
        vec![768.0, 768.0, 770.0, 772.0, 770.0, 775.0, 778.0, 776.0, 780.0, 782.0]
    }
}

impl GPUMonitor {
    pub fn get_current_utilization(&self) -> f32 {
        65.5
    }

    pub fn get_memory_usage(&self) -> f32 {
        1024.0
    }
}

impl CPUMonitor {
    pub fn get_per_core_series(&self) -> Vec<DataSeries> {
        vec![
            DataSeries {
                name: "Core 0".to_string(),
                values: vec![45.2],
                color: Some(Color::new(0.5, 0.7, 1.0, 1.0)),
            },
            DataSeries {
                name: "Core 1".to_string(),
                values: vec![38.7],
                color: Some(Color::new(0.5, 0.7, 1.0, 1.0)),
            },
            // Additional cores...
        ]
    }
}

// Supporting components

#[derive(Debug)]
pub struct OptimizationVisualizer;

impl OptimizationVisualizer {
    pub fn new() -> Self { Self }

    pub fn update(&mut self, _metrics: &PerformanceMetrics) -> RobinResult<()> {
        Ok(())
    }

    pub fn get_culling_efficiency(&self) -> f32 { 0.92 }
    pub fn get_visible_chunks(&self) -> u32 { 184 }
    pub fn get_total_chunks(&self) -> u32 { 200 }
    pub fn get_vertex_reduction(&self) -> f32 { 0.75 }
    pub fn get_vertex_count(&self) -> u32 { 125000 }
    pub fn get_draw_calls(&self) -> u32 { 245 }
    pub fn get_batched_calls(&self) -> u32 { 82 }
}

#[derive(Debug)]
pub struct StatisticsPanel;

impl StatisticsPanel {
    pub fn new() -> Self { Self }

    pub fn update(&mut self, _metrics: &PerformanceMetrics) -> RobinResult<()> {
        Ok(())
    }

    pub fn get_update_time(&self) -> f32 { 3.2 }
    pub fn get_render_time(&self) -> f32 { 12.5 }
    pub fn get_present_time(&self) -> f32 { 0.9 }
}

#[derive(Debug)]
pub struct AlertSystem {
    active_alerts: Vec<PerformanceAlert>,
}

impl AlertSystem {
    pub fn new() -> Self {
        Self {
            active_alerts: Vec::new(),
        }
    }

    pub fn check_alerts(&mut self, metrics: &PerformanceMetrics) -> RobinResult<()> {
        self.active_alerts.clear();

        if metrics.fps < 30.0 {
            self.active_alerts.push(PerformanceAlert {
                level: AlertLevel::Warning,
                message: format!("Low FPS: {:.1}", metrics.fps),
            });
        }

        if metrics.memory_used_mb > 1500.0 {
            self.active_alerts.push(PerformanceAlert {
                level: AlertLevel::Critical,
                message: format!("High memory usage: {:.0}MB", metrics.memory_used_mb),
            });
        }

        Ok(())
    }

    pub fn get_active_alert_components(&self) -> Vec<UIComponent> {
        Vec::new() // Simplified
    }
}

#[derive(Debug)]
pub struct DashboardLayout {
    position: DashboardPosition,
    size: DashboardSize,
    compact_mode: bool,
}

impl DashboardLayout {
    pub fn new(config: &DashboardConfig) -> Self {
        Self {
            position: config.position,
            size: config.size,
            compact_mode: config.compact_mode,
        }
    }

    pub fn get_panel_bounds(&self) -> Rect {
        let (width, height) = match self.size {
            DashboardSize::Compact => (250.0, 150.0),
            DashboardSize::Normal => (400.0, 300.0),
            DashboardSize::Extended => (600.0, 450.0),
            DashboardSize::Custom(w, h) => (w, h),
        };

        let (x, y) = match self.position {
            DashboardPosition::TopRight => (1920.0 - width - 20.0, 20.0),
            DashboardPosition::TopLeft => (20.0, 20.0),
            DashboardPosition::BottomRight => (1920.0 - width - 20.0, 1080.0 - height - 20.0),
            DashboardPosition::BottomLeft => (20.0, 1080.0 - height - 20.0),
            DashboardPosition::Custom(x, y) => (x, y),
        };

        Rect::new(x, y, width, height)
    }

    pub fn get_title_bounds(&self) -> Rect {
        let panel = self.get_panel_bounds();
        Rect::new(panel.x, panel.y + 10.0, panel.width, 30.0)
    }

    pub fn get_fps_graph_bounds(&self) -> Rect {
        let panel = self.get_panel_bounds();
        Rect::new(panel.x + 10.0, panel.y + 50.0, panel.width * 0.45, 80.0)
    }

    pub fn get_memory_graph_bounds(&self) -> Rect {
        let panel = self.get_panel_bounds();
        Rect::new(panel.x + panel.width * 0.5 + 5.0, panel.y + 50.0, panel.width * 0.45, 80.0)
    }

    pub fn get_gpu_bounds(&self) -> Rect {
        let panel = self.get_panel_bounds();
        Rect::new(panel.x + 10.0, panel.y + 140.0, panel.width * 0.45, 30.0)
    }

    pub fn get_cpu_bounds(&self) -> Rect {
        let panel = self.get_panel_bounds();
        Rect::new(panel.x + panel.width * 0.5 + 5.0, panel.y + 140.0, panel.width * 0.45, 30.0)
    }

    pub fn get_optimization_text_bounds(&self, index: usize) -> Rect {
        let panel = self.get_panel_bounds();
        Rect::new(panel.x + 10.0, panel.y + 180.0 + (index as f32 * 20.0), panel.width - 20.0, 20.0)
    }

    pub fn get_statistics_bounds(&self) -> Rect {
        let panel = self.get_panel_bounds();
        Rect::new(panel.x + 10.0, panel.y + 250.0, panel.width - 20.0, 40.0)
    }

    pub fn update_position(&mut self, position: DashboardPosition) {
        self.position = position;
    }

    pub fn set_compact_mode(&mut self, compact: bool) {
        self.compact_mode = compact;
    }
}

// Data structures

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub frame_time_ms: f32,
    pub fps: f32,
    pub memory_used_mb: f32,
    pub memory_allocated_mb: f32,
    pub gpu_utilization: f32,
    pub gpu_memory_mb: f32,
    pub cpu_utilization: f32,
    pub cpu_cores: Vec<f32>,
    pub draw_calls: u32,
    pub vertices_rendered: u32,
    pub triangles_rendered: u32,
    pub texture_memory_mb: f32,
    pub culling_efficiency: f32,
    pub vertex_reduction: f32,
    pub timestamp: Instant,
}

#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub level: AlertLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}