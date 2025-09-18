use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use crate::engine::ui::design_system::{DesignSystem, ColorPalette};
use crate::engine::ui::modern_architecture::{Theme, Color, FontWeight, FontVariant, ThemeColors, ThemeTypography, ThemeSpacing, ThemeBorders, ThemeShadows, ThemeAnimations, FontStyle, ShadowStyle};
use crate::engine::math::Vec2;
use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};

/// Theme engine that manages runtime theme switching and provides theme context to components
pub struct ThemeEngine {
    design_system: Arc<RwLock<DesignSystem>>,
    theme_listeners: Vec<Arc<dyn Fn(&Theme) + Send + Sync>>,
    custom_themes: HashMap<String, Theme>,
    theme_overrides: HashMap<String, ThemeOverride>,
    system_preference: SystemThemePreference,
    auto_switch_enabled: bool,
}

/// System theme preference detection
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SystemThemePreference {
    Light,
    Dark,
    HighContrast,
    Auto,
}

/// Theme override for specific components or sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeOverride {
    pub component_id: Option<String>,
    pub section: Option<String>,
    pub overrides: HashMap<String, ThemeValue>,
}

/// Individual theme value that can be overridden
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeValue {
    Color(Color),
    Spacing(f32),
    FontSize(f32),
    BorderRadius(f32),
    Shadow(ShadowValue),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowValue {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

/// Theme context provided to components during rendering
pub struct ThemeContext {
    pub current_theme: Theme,
    pub palette: ColorPalette,
    pub spacing_multiplier: f32,
    pub animation_speed: f32,
    pub reduced_motion: bool,
}

impl ThemeEngine {
    pub fn new(design_system: Arc<RwLock<DesignSystem>>) -> Self {
        Self {
            design_system,
            theme_listeners: Vec::new(),
            custom_themes: HashMap::new(),
            theme_overrides: HashMap::new(),
            system_preference: SystemThemePreference::Auto,
            auto_switch_enabled: true,
        }
    }

    /// Switch to a specific theme by name
    pub fn switch_theme(&mut self, theme_name: &str) -> RobinResult<()> {
        // First check custom themes
        if let Some(custom_theme) = self.custom_themes.get(theme_name) {
            self.apply_theme(custom_theme.clone())?;
        } else {
            // Fall back to design system themes
            let mut ds = self.design_system.write().unwrap();
            ds.set_active_theme(theme_name).map_err(|e| crate::engine::error::RobinError::Other(e))?;

            let theme = ds.get_active_theme().clone();
            drop(ds); // Release lock before notifying listeners
            self.notify_listeners(&theme);
        }

        Ok(())
    }

    /// Toggle between light and dark themes
    pub fn toggle_theme(&mut self) -> RobinResult<()> {
        let ds = self.design_system.read().unwrap();
        let current_theme = ds.get_active_theme_name();

        let next_theme = match current_theme {
            "light" => "dark",
            "dark" => "light",
            _ => "dark", // Default to dark if current theme is custom
        };

        drop(ds); // Release read lock before switching
        self.switch_theme(next_theme)
    }

    /// Register a custom theme
    pub fn register_custom_theme(&mut self, name: String, theme: Theme) {
        self.custom_themes.insert(name, theme);
    }

    /// Apply theme overrides for specific components or sections
    pub fn apply_override(&mut self, override_key: String, theme_override: ThemeOverride) {
        self.theme_overrides.insert(override_key, theme_override);
    }

    /// Remove a theme override
    pub fn remove_override(&mut self, override_key: &str) {
        self.theme_overrides.remove(override_key);
    }

    /// Get the current theme context for rendering
    pub fn get_context(&self) -> ThemeContext {
        let ds = self.design_system.read().unwrap();
        let current_theme = ds.get_active_theme().clone();

        ThemeContext {
            current_theme: current_theme.clone(),
            palette: ds.get_palette().clone(),
            spacing_multiplier: 1.0,
            animation_speed: 1.0,
            reduced_motion: false, // Default to false for now
        }
    }

    /// Enable or disable automatic theme switching based on system preferences
    pub fn set_auto_switch(&mut self, enabled: bool) {
        self.auto_switch_enabled = enabled;
        if enabled {
            self.sync_with_system();
        }
    }

    /// Subscribe to theme changes
    pub fn on_theme_change<F>(&mut self, listener: F)
    where
        F: Fn(&Theme) + Send + Sync + 'static,
    {
        self.theme_listeners.push(Arc::new(listener));
    }

    /// Apply a theme directly
    fn apply_theme(&mut self, theme: Theme) -> RobinResult<()> {
        let mut ds = self.design_system.write().unwrap();
        // Store the theme in the design system
        let theme_name = format!("custom_{}", self.custom_themes.len());
        ds.register_theme(&theme_name, theme.clone());
        ds.set_active_theme(&theme_name).map_err(|e| crate::engine::error::RobinError::Other(e))?;

        self.notify_listeners(&theme);
        Ok(())
    }

    /// Notify all listeners of theme change
    fn notify_listeners(&self, theme: &Theme) {
        for listener in &self.theme_listeners {
            listener(theme);
        }
    }

    /// Check if reduced motion is preferred
    fn check_reduced_motion(&self) -> bool {
        // This would typically check system preferences
        // For now, return false as default
        false
    }

    /// Sync theme with system preferences
    fn sync_with_system(&mut self) {
        if !self.auto_switch_enabled {
            return;
        }

        let system_theme = self.detect_system_theme();
        let theme_name = match system_theme {
            SystemThemePreference::Light => "light",
            SystemThemePreference::Dark => "dark",
            SystemThemePreference::HighContrast => "high_contrast",
            SystemThemePreference::Auto => {
                // Determine based on time of day or other factors
                if self.is_dark_mode_time() {
                    "dark"
                } else {
                    "light"
                }
            }
        };

        let _ = self.switch_theme(theme_name);
    }

    /// Detect the system theme preference
    fn detect_system_theme(&self) -> SystemThemePreference {
        // This would typically use platform-specific APIs
        // For now, return Auto as default
        SystemThemePreference::Auto
    }

    /// Check if it's dark mode time (simple implementation)
    fn is_dark_mode_time(&self) -> bool {
        use chrono::{Local, Timelike};
        let hour = Local::now().hour();
        hour < 6 || hour >= 20 // Dark mode between 8 PM and 6 AM
    }

    /// Create a theme from a color scheme
    pub fn create_theme_from_colors(
        &self,
        name: &str,
        primary: Color,
        secondary: Color,
        background: Color,
        text: Color,
    ) -> Theme {
        Theme {
            name: name.to_string(),
            colors: ThemeColors {
                primary,
                secondary,
                accent: secondary,
                background,
                surface: background.lighten(0.05),
                text,
                text_secondary: text.with_alpha(0.7),
                error: Color::from_hex("#ef4444").unwrap(),
                warning: Color::from_hex("#f59e0b").unwrap(),
                success: Color::from_hex("#10b981").unwrap(),
                info: Color::from_hex("#3b82f6").unwrap(),
            },
            typography: ThemeTypography {
                font_family: "Inter".to_string(),
                heading1: FontStyle {
                    family: "Inter".to_string(),
                    size: 32.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                heading2: FontStyle {
                    family: "Inter".to_string(),
                    size: 24.0,
                    weight: FontWeight::Bold,
                    style: FontVariant::Normal,
                },
                heading3: FontStyle {
                    family: "Inter".to_string(),
                    size: 20.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
                body: FontStyle {
                    family: "Inter".to_string(),
                    size: 16.0,
                    weight: FontWeight::Regular,
                    style: FontVariant::Normal,
                },
                caption: FontStyle {
                    family: "Inter".to_string(),
                    size: 14.0,
                    weight: FontWeight::Regular,
                    style: FontVariant::Normal,
                },
                button: FontStyle {
                    family: "Inter".to_string(),
                    size: 16.0,
                    weight: FontWeight::Medium,
                    style: FontVariant::Normal,
                },
            },
            spacing: ThemeSpacing {
                xs: 4.0,
                sm: 8.0,
                md: 16.0,
                lg: 24.0,
                xl: 32.0,
                xxl: 48.0,
            },
            borders: ThemeBorders {
                radius_sm: 4.0,
                radius_md: 8.0,
                radius_lg: 12.0,
                width_thin: 1.0,
                width_medium: 2.0,
                width_thick: 4.0,
            },
            shadows: ThemeShadows {
                sm: ShadowStyle {
                    offset: Vec2::new(0.0, 1.0),
                    blur: 2.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.1),
                },
                md: ShadowStyle {
                    offset: Vec2::new(0.0, 4.0),
                    blur: 8.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.1),
                },
                lg: ShadowStyle {
                    offset: Vec2::new(0.0, 8.0),
                    blur: 16.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.1),
                },
                xl: ShadowStyle {
                    offset: Vec2::new(0.0, 16.0),
                    blur: 32.0,
                    spread: 0.0,
                    color: Color::new(0.0, 0.0, 0.0, 0.15),
                },
            },
            animations: ThemeAnimations {
                duration_fast: 0.15,
                duration_normal: 0.3,
                duration_slow: 0.5,
                easing_linear: "linear".to_string(),
                easing_ease_in: "ease-in".to_string(),
                easing_ease_out: "ease-out".to_string(),
                easing_ease_in_out: "ease-in-out".to_string(),
                easing_spring: "cubic-bezier(0.68, -0.55, 0.265, 1.55)".to_string(),
            },
        }
    }

    /// Export current theme as JSON
    pub fn export_theme(&self) -> RobinResult<String> {
        let ds = self.design_system.read().unwrap();
        let theme = ds.get_active_theme();

        Ok(serde_json::to_string_pretty(theme)
            .map_err(|e| crate::engine::error::RobinError::Other(e.to_string()))?)
    }

    /// Import theme from JSON
    pub fn import_theme(&mut self, json: &str) -> RobinResult<()> {
        let theme: Theme = serde_json::from_str(json)?;
        self.register_custom_theme(theme.name.clone(), theme);
        Ok(())
    }
}

/// Theme interpolation for smooth transitions
pub struct ThemeInterpolator {
    from: Theme,
    to: Theme,
    duration: f32,
    elapsed: f32,
}

impl ThemeInterpolator {
    pub fn new(from: Theme, to: Theme, duration: f32) -> Self {
        Self {
            from,
            to,
            duration,
            elapsed: 0.0,
        }
    }

    pub fn update(&mut self, delta_time: f32) -> bool {
        self.elapsed += delta_time;
        self.elapsed >= self.duration
    }

    pub fn get_interpolated(&self) -> Theme {
        let t = (self.elapsed / self.duration).min(1.0);
        self.interpolate_themes(&self.from, &self.to, t)
    }

    fn interpolate_themes(&self, from: &Theme, to: &Theme, t: f32) -> Theme {
        Theme {
            name: if t < 0.5 { from.name.clone() } else { to.name.clone() },
            colors: ThemeColors {
                primary: from.colors.primary.lerp(&to.colors.primary, t),
                secondary: from.colors.secondary.lerp(&to.colors.secondary, t),
                accent: from.colors.accent.lerp(&to.colors.accent, t),
                background: from.colors.background.lerp(&to.colors.background, t),
                surface: from.colors.surface.lerp(&to.colors.surface, t),
                text: from.colors.text.lerp(&to.colors.text, t),
                text_secondary: from.colors.text_secondary.lerp(&to.colors.text_secondary, t),
                error: from.colors.error.lerp(&to.colors.error, t),
                warning: from.colors.warning.lerp(&to.colors.warning, t),
                success: from.colors.success.lerp(&to.colors.success, t),
                info: from.colors.info.lerp(&to.colors.info, t),
            },
            typography: from.typography.clone(), // Typography doesn't interpolate easily
            spacing: ThemeSpacing {
                xs: from.spacing.xs + (to.spacing.xs - from.spacing.xs) * t,
                sm: from.spacing.sm + (to.spacing.sm - from.spacing.sm) * t,
                md: from.spacing.md + (to.spacing.md - from.spacing.md) * t,
                lg: from.spacing.lg + (to.spacing.lg - from.spacing.lg) * t,
                xl: from.spacing.xl + (to.spacing.xl - from.spacing.xl) * t,
                xxl: from.spacing.xxl + (to.spacing.xxl - from.spacing.xxl) * t,
            },
            borders: ThemeBorders {
                radius_sm: from.borders.radius_sm + (to.borders.radius_sm - from.borders.radius_sm) * t,
                radius_md: from.borders.radius_md + (to.borders.radius_md - from.borders.radius_md) * t,
                radius_lg: from.borders.radius_lg + (to.borders.radius_lg - from.borders.radius_lg) * t,
                width_thin: from.borders.width_thin + (to.borders.width_thin - from.borders.width_thin) * t,
                width_medium: from.borders.width_medium + (to.borders.width_medium - from.borders.width_medium) * t,
                width_thick: from.borders.width_thick + (to.borders.width_thick - from.borders.width_thick) * t,
            },
            shadows: ThemeShadows {
                sm: ShadowStyle {
                    offset: Vec2::new(
                        from.shadows.sm.offset.x + (to.shadows.sm.offset.x - from.shadows.sm.offset.x) * t,
                        from.shadows.sm.offset.y + (to.shadows.sm.offset.y - from.shadows.sm.offset.y) * t,
                    ),
                    blur: from.shadows.sm.blur + (to.shadows.sm.blur - from.shadows.sm.blur) * t,
                    spread: from.shadows.sm.spread + (to.shadows.sm.spread - from.shadows.sm.spread) * t,
                    color: from.shadows.sm.color.lerp(&to.shadows.sm.color, t),
                },
                md: ShadowStyle {
                    offset: Vec2::new(
                        from.shadows.md.offset.x + (to.shadows.md.offset.x - from.shadows.md.offset.x) * t,
                        from.shadows.md.offset.y + (to.shadows.md.offset.y - from.shadows.md.offset.y) * t,
                    ),
                    blur: from.shadows.md.blur + (to.shadows.md.blur - from.shadows.md.blur) * t,
                    spread: from.shadows.md.spread + (to.shadows.md.spread - from.shadows.md.spread) * t,
                    color: from.shadows.md.color.lerp(&to.shadows.md.color, t),
                },
                lg: ShadowStyle {
                    offset: Vec2::new(
                        from.shadows.lg.offset.x + (to.shadows.lg.offset.x - from.shadows.lg.offset.x) * t,
                        from.shadows.lg.offset.y + (to.shadows.lg.offset.y - from.shadows.lg.offset.y) * t,
                    ),
                    blur: from.shadows.lg.blur + (to.shadows.lg.blur - from.shadows.lg.blur) * t,
                    spread: from.shadows.lg.spread + (to.shadows.lg.spread - from.shadows.lg.spread) * t,
                    color: from.shadows.lg.color.lerp(&to.shadows.lg.color, t),
                },
                xl: ShadowStyle {
                    offset: Vec2::new(
                        from.shadows.xl.offset.x + (to.shadows.xl.offset.x - from.shadows.xl.offset.x) * t,
                        from.shadows.xl.offset.y + (to.shadows.xl.offset.y - from.shadows.xl.offset.y) * t,
                    ),
                    blur: from.shadows.xl.blur + (to.shadows.xl.blur - from.shadows.xl.blur) * t,
                    spread: from.shadows.xl.spread + (to.shadows.xl.spread - from.shadows.xl.spread) * t,
                    color: from.shadows.xl.color.lerp(&to.shadows.xl.color, t),
                },
            },
            animations: ThemeAnimations {
                duration_fast: from.animations.duration_fast + (to.animations.duration_fast - from.animations.duration_fast) * t,
                duration_normal: from.animations.duration_normal + (to.animations.duration_normal - from.animations.duration_normal) * t,
                duration_slow: from.animations.duration_slow + (to.animations.duration_slow - from.animations.duration_slow) * t,
                easing_linear: from.animations.easing_linear.clone(),
                easing_ease_in: from.animations.easing_ease_in.clone(),
                easing_ease_out: from.animations.easing_ease_out.clone(),
                easing_ease_in_out: from.animations.easing_ease_in_out.clone(),
                easing_spring: from.animations.easing_spring.clone(),
            },
        }
    }
}

/// Automatic theme scheduling based on time
pub struct ThemeScheduler {
    schedules: Vec<ThemeSchedule>,
    enabled: bool,
}

#[derive(Debug, Clone)]
pub struct ThemeSchedule {
    pub theme_name: String,
    pub start_hour: u32,
    pub start_minute: u32,
    pub end_hour: u32,
    pub end_minute: u32,
    pub days: Vec<chrono::Weekday>,
}

impl ThemeScheduler {
    pub fn new() -> Self {
        Self {
            schedules: Vec::new(),
            enabled: false,
        }
    }

    pub fn add_schedule(&mut self, schedule: ThemeSchedule) {
        self.schedules.push(schedule);
    }

    pub fn get_scheduled_theme(&self) -> Option<String> {
        if !self.enabled {
            return None;
        }

        use chrono::{Local, Timelike, Datelike};
        let now = Local::now();
        let current_hour = now.hour();
        let current_minute = now.minute();
        let current_day = now.weekday();

        for schedule in &self.schedules {
            if !schedule.days.contains(&current_day) {
                continue;
            }

            let start_minutes = schedule.start_hour * 60 + schedule.start_minute;
            let end_minutes = schedule.end_hour * 60 + schedule.end_minute;
            let current_minutes = current_hour * 60 + current_minute;

            if start_minutes <= end_minutes {
                // Schedule doesn't cross midnight
                if current_minutes >= start_minutes && current_minutes < end_minutes {
                    return Some(schedule.theme_name.clone());
                }
            } else {
                // Schedule crosses midnight
                if current_minutes >= start_minutes || current_minutes < end_minutes {
                    return Some(schedule.theme_name.clone());
                }
            }
        }

        None
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

// Color manipulation extensions for theme operations
impl Color {
    pub fn lerp(&self, other: &Color, t: f32) -> Color {
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    pub fn lighten(&self, amount: f32) -> Color {
        Color {
            r: (self.r + amount).min(1.0),
            g: (self.g + amount).min(1.0),
            b: (self.b + amount).min(1.0),
            a: self.a,
        }
    }

    pub fn darken(&self, amount: f32) -> Color {
        Color {
            r: (self.r - amount).max(0.0),
            g: (self.g - amount).max(0.0),
            b: (self.b - amount).max(0.0),
            a: self.a,
        }
    }

    pub fn with_alpha(&self, alpha: f32) -> Color {
        Color {
            r: self.r,
            g: self.g,
            b: self.b,
            a: alpha,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_switching() {
        let design_system = Arc::new(RwLock::new(DesignSystem::new()));
        let mut engine = ThemeEngine::new(design_system);

        assert!(engine.switch_theme("dark").is_ok());
        assert!(engine.switch_theme("light").is_ok());
    }

    #[test]
    fn test_theme_toggle() {
        let design_system = Arc::new(RwLock::new(DesignSystem::new()));
        let mut engine = ThemeEngine::new(design_system);

        engine.switch_theme("light").unwrap();
        engine.toggle_theme().unwrap();
        // Should now be dark theme
    }

    #[test]
    fn test_custom_theme() {
        let design_system = Arc::new(RwLock::new(DesignSystem::new()));
        let mut engine = ThemeEngine::new(design_system);

        let custom = engine.create_theme_from_colors(
            "custom",
            Color::from_hex("#ff0000").unwrap(),
            Color::from_hex("#00ff00").unwrap(),
            Color::from_hex("#000000").unwrap(),
            Color::from_hex("#ffffff").unwrap(),
        );

        engine.register_custom_theme("custom".to_string(), custom);
        assert!(engine.switch_theme("custom").is_ok());
    }

    #[test]
    fn test_theme_interpolation() {
        let dark = Theme::default();
        let light = Theme {
            name: "light".to_string(),
            background: Color::from_hex("#ffffff").unwrap(),
            ..Theme::default()
        };

        let mut interpolator = ThemeInterpolator::new(dark, light, 1.0);
        interpolator.update(0.5);

        let mid = interpolator.get_interpolated();
        // Background should be somewhere between black and white
        assert!(mid.background.r > 0.0 && mid.background.r < 1.0);
    }

    #[test]
    fn test_theme_scheduler() {
        let mut scheduler = ThemeScheduler::new();
        scheduler.add_schedule(ThemeSchedule {
            theme_name: "dark".to_string(),
            start_hour: 20,
            start_minute: 0,
            end_hour: 6,
            end_minute: 0,
            days: vec![
                chrono::Weekday::Mon,
                chrono::Weekday::Tue,
                chrono::Weekday::Wed,
                chrono::Weekday::Thu,
                chrono::Weekday::Fri,
                chrono::Weekday::Sat,
                chrono::Weekday::Sun,
            ],
        });

        scheduler.enable();
        // Would return Some("dark") if current time is between 8 PM and 6 AM
    }
}