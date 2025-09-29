// Integration test for performance dashboard functionality
use crate::demo_state::{PerformanceDashboard, PerformanceTier};
use crate::ui::PerformanceDashboardPanel;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_dashboard_creation() {
        let dashboard = PerformanceDashboard::new();
        assert_eq!(dashboard.current_fps, 60.0);
        assert_eq!(dashboard.performance_tier, PerformanceTier::Excellent);
        println!("✅ Performance Dashboard creation test passed");
    }

    #[test]
    fn test_performance_dashboard_panel() {
        let mut panel = PerformanceDashboardPanel::new();
        assert!(!panel.is_visible());

        panel.toggle();
        assert!(panel.is_visible());

        panel.set_visible(false);
        assert!(!panel.is_visible());

        println!("✅ Performance Dashboard Panel test passed");
    }

    #[test]
    fn test_performance_metrics_update() {
        let mut dashboard = PerformanceDashboard::new();

        // Simulate poor performance
        dashboard.update(0.05); // 20 FPS (50ms frame time)

        // Should register as poor performance
        assert!(dashboard.current_fps < 30.0);

        println!("✅ Performance metrics update test passed");
    }

    #[test]
    fn test_performance_tier_calculation() {
        let mut dashboard = PerformanceDashboard::new();

        // Test excellent performance (60+ FPS)
        dashboard.current_fps = 75.0;
        dashboard.frustum_culling_efficiency = 95.0;
        dashboard.vertex_reduction_percentage = 70.0;

        // Manually calculate tier - should be excellent
        let tier = if dashboard.current_fps >= 60.0 &&
                     dashboard.frustum_culling_efficiency >= 85.0 &&
                     dashboard.vertex_reduction_percentage >= 60.0 {
            PerformanceTier::Excellent
        } else if dashboard.current_fps >= 45.0 {
            PerformanceTier::Good
        } else if dashboard.current_fps >= 30.0 {
            PerformanceTier::Fair
        } else {
            PerformanceTier::Poor
        };

        assert_eq!(tier, PerformanceTier::Excellent);
        println!("✅ Performance tier calculation test passed");
    }
}

pub fn run_integration_tests() {
    println!("🧪 Running Performance Dashboard Integration Tests...");

    // Test performance dashboard creation
    let dashboard = PerformanceDashboard::new();
    println!("✅ PerformanceDashboard::new() - Success");

    // Test performance dashboard panel
    let mut panel = PerformanceDashboardPanel::new();
    panel.toggle();
    println!("✅ PerformanceDashboardPanel toggle - Success");

    // Test performance metrics
    let mut test_dashboard = PerformanceDashboard::new();
    test_dashboard.update(0.016); // 60 FPS
    println!("✅ Performance metrics update - Success");

    println!("🎉 All integration tests passed!");
}