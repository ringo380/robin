use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::engine::error::RobinResult;

#[derive(Debug, Clone)]
pub struct ProfilingReport {
    pub total_frame_time: Duration,
    pub sections: HashMap<String, ProfileSection>,
    pub call_counts: HashMap<String, u32>,
    pub memory_allocations: HashMap<String, u64>,
    pub hot_spots: Vec<HotSpot>,
    pub frame_number: u64,
}

#[derive(Debug, Clone)]
pub struct ProfileSection {
    pub name: String,
    pub total_time: Duration,
    pub average_time: Duration,
    pub min_time: Duration,
    pub max_time: Duration,
    pub call_count: u32,
    pub percentage_of_frame: f32,
}

#[derive(Debug, Clone)]
pub struct HotSpot {
    pub function_name: String,
    pub time_spent: Duration,
    pub percentage_of_frame: f32,
    pub call_count: u32,
    pub severity: HotSpotSeverity,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HotSpotSeverity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct Profiler {
    enabled: bool,
    frame_start: Option<Instant>,
    current_frame: u64,
    section_timers: HashMap<String, SectionTimer>,
    frame_history: Vec<ProfilingReport>,
    max_history_size: usize,
}

struct SectionTimer {
    start_time: Option<Instant>,
    total_time: Duration,
    call_count: u32,
    min_time: Duration,
    max_time: Duration,
}

impl Default for SectionTimer {
    fn default() -> Self {
        Self {
            start_time: None,
            total_time: Duration::new(0, 0),
            call_count: 0,
            min_time: Duration::new(u64::MAX, 0),
            max_time: Duration::new(0, 0),
        }
    }
}

impl Profiler {
    pub fn new(enabled: bool) -> RobinResult<Self> {
        Ok(Self {
            enabled,
            frame_start: None,
            current_frame: 0,
            section_timers: HashMap::new(),
            frame_history: Vec::new(),
            max_history_size: 300, // Keep 5 seconds of history at 60 FPS
        })
    }
    
    pub fn set_enabled(&mut self, enabled: bool) -> RobinResult<()> {
        self.enabled = enabled;
        if !enabled {
            self.section_timers.clear();
        }
        Ok(())
    }
    
    pub fn begin_frame(&mut self) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        self.frame_start = Some(Instant::now());
        self.current_frame += 1;
        
        // Reset section timers for new frame
        for timer in self.section_timers.values_mut() {
            timer.total_time = Duration::new(0, 0);
            timer.call_count = 0;
            timer.min_time = Duration::new(u64::MAX, 0);
            timer.max_time = Duration::new(0, 0);
        }
        
        Ok(())
    }
    
    pub fn end_frame(&mut self, frame_time: Duration) -> RobinResult<()> {
        if !self.enabled || self.frame_start.is_none() {
            return Ok(());
        }
        
        // Generate profiling report for this frame
        let mut sections = HashMap::new();
        let mut hot_spots = Vec::new();
        
        for (name, timer) in &self.section_timers {
            if timer.call_count == 0 {
                continue;
            }
            
            let section = ProfileSection {
                name: name.clone(),
                total_time: timer.total_time,
                average_time: timer.total_time / timer.call_count,
                min_time: timer.min_time,
                max_time: timer.max_time,
                call_count: timer.call_count,
                percentage_of_frame: if frame_time.as_nanos() > 0 {
                    (timer.total_time.as_nanos() as f32 / frame_time.as_nanos() as f32) * 100.0
                } else {
                    0.0
                },
            };
            
            // Identify hot spots
            if section.percentage_of_frame > 20.0 {
                hot_spots.push(HotSpot {
                    function_name: name.clone(),
                    time_spent: timer.total_time,
                    percentage_of_frame: section.percentage_of_frame,
                    call_count: timer.call_count,
                    severity: if section.percentage_of_frame > 40.0 {
                        HotSpotSeverity::Critical
                    } else if section.percentage_of_frame > 30.0 {
                        HotSpotSeverity::High
                    } else {
                        HotSpotSeverity::Medium
                    },
                });
            }
            
            sections.insert(name.clone(), section);
        }
        
        // Sort hot spots by severity and time
        hot_spots.sort_by(|a, b| {
            b.percentage_of_frame.partial_cmp(&a.percentage_of_frame).unwrap()
        });
        
        let report = ProfilingReport {
            total_frame_time: frame_time,
            sections,
            call_counts: self.section_timers.iter()
                .map(|(name, timer)| (name.clone(), timer.call_count))
                .collect(),
            memory_allocations: HashMap::new(), // Would be populated by memory profiler
            hot_spots,
            frame_number: self.current_frame,
        };
        
        // Store in history
        self.frame_history.push(report);
        if self.frame_history.len() > self.max_history_size {
            self.frame_history.remove(0);
        }
        
        Ok(())
    }
    
    pub fn begin_section(&mut self, name: &str) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let timer = self.section_timers.entry(name.to_string()).or_default();
        timer.start_time = Some(Instant::now());
        Ok(())
    }
    
    pub fn end_section(&mut self, name: &str) -> RobinResult<()> {
        if !self.enabled {
            return Ok(());
        }
        
        let now = Instant::now();
        if let Some(timer) = self.section_timers.get_mut(name) {
            if let Some(start_time) = timer.start_time {
                let elapsed = now - start_time;
                timer.total_time += elapsed;
                timer.call_count += 1;
                
                if elapsed < timer.min_time {
                    timer.min_time = elapsed;
                }
                if elapsed > timer.max_time {
                    timer.max_time = elapsed;
                }
                
                timer.start_time = None;
            }
        }
        
        Ok(())
    }
    
    pub fn get_report(&self) -> Option<ProfilingReport> {
        if self.enabled && !self.frame_history.is_empty() {
            Some(self.frame_history.last().unwrap().clone())
        } else {
            None
        }
    }
    
    pub fn get_average_report(&self, frames: usize) -> Option<ProfilingReport> {
        if !self.enabled || self.frame_history.is_empty() {
            return None;
        }
        
        let frames_to_average = frames.min(self.frame_history.len());
        let recent_reports: Vec<&ProfilingReport> = self.frame_history
            .iter()
            .rev()
            .take(frames_to_average)
            .collect();
        
        if recent_reports.is_empty() {
            return None;
        }
        
        // Calculate averages
        let avg_frame_time = recent_reports.iter()
            .map(|r| r.total_frame_time)
            .sum::<Duration>() / frames_to_average as u32;
        
        let mut averaged_sections = HashMap::new();
        let mut all_section_names = std::collections::HashSet::new();
        
        for report in &recent_reports {
            for name in report.sections.keys() {
                all_section_names.insert(name.clone());
            }
        }
        
        for section_name in all_section_names {
            let mut total_time = Duration::new(0, 0);
            let mut total_calls = 0;
            let mut count = 0;
            
            for report in &recent_reports {
                if let Some(section) = report.sections.get(&section_name) {
                    total_time += section.total_time;
                    total_calls += section.call_count;
                    count += 1;
                }
            }
            
            if count > 0 {
                averaged_sections.insert(section_name.clone(), ProfileSection {
                    name: section_name.clone(),
                    total_time: total_time / count as u32,
                    average_time: if total_calls > 0 { total_time / total_calls } else { Duration::new(0, 0) },
                    min_time: Duration::new(0, 0), // Would need more complex calculation
                    max_time: Duration::new(0, 0), // Would need more complex calculation
                    call_count: total_calls / count as u32,
                    percentage_of_frame: if avg_frame_time.as_nanos() > 0 {
                        ((total_time / count as u32).as_nanos() as f32 / avg_frame_time.as_nanos() as f32) * 100.0
                    } else {
                        0.0
                    },
                });
            }
        }
        
        Some(ProfilingReport {
            total_frame_time: avg_frame_time,
            sections: averaged_sections,
            call_counts: HashMap::new(),
            memory_allocations: HashMap::new(),
            hot_spots: Vec::new(),
            frame_number: self.current_frame,
        })
    }
    
    pub fn get_performance_trends(&self) -> PerformanceTrends {
        if self.frame_history.len() < 2 {
            return PerformanceTrends::default();
        }
        
        let recent_count = 30.min(self.frame_history.len());
        let recent_frames: Vec<&ProfilingReport> = self.frame_history
            .iter()
            .rev()
            .take(recent_count)
            .collect();
        
        let mut frame_times: Vec<f32> = recent_frames.iter()
            .map(|r| r.total_frame_time.as_secs_f32())
            .collect();
        frame_times.reverse(); // Chronological order
        
        // Calculate trend (simple linear regression)
        let n = frame_times.len() as f32;
        let sum_x: f32 = (0..frame_times.len()).map(|i| i as f32).sum();
        let sum_y: f32 = frame_times.iter().sum();
        let sum_xy: f32 = frame_times.iter().enumerate()
            .map(|(i, &y)| i as f32 * y)
            .sum();
        let sum_x2: f32 = (0..frame_times.len())
            .map(|i| (i as f32).powi(2))
            .sum();
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        
        let trend_direction = if slope.abs() < 0.0001 {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Degrading
        } else {
            TrendDirection::Improving
        };
        
        let variance = frame_times.iter()
            .map(|&x| (x - sum_y / n).powi(2))
            .sum::<f32>() / n;
        
        PerformanceTrends {
            frame_time_trend: trend_direction,
            average_frame_time: sum_y / n,
            frame_time_variance: variance,
            frames_analyzed: frame_times.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceTrends {
    pub frame_time_trend: TrendDirection,
    pub average_frame_time: f32,
    pub frame_time_variance: f32,
    pub frames_analyzed: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
}

impl Default for PerformanceTrends {
    fn default() -> Self {
        Self {
            frame_time_trend: TrendDirection::Stable,
            average_frame_time: 0.0,
            frame_time_variance: 0.0,
            frames_analyzed: 0,
        }
    }
}

// Convenience macro for profiling sections
#[macro_export]
macro_rules! profile_section {
    ($profiler:expr, $name:expr, $code:block) => {
        {
            let _ = $profiler.begin_section($name);
            let result = $code;
            let _ = $profiler.end_section($name);
            result
        }
    };
}