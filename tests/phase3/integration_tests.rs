use robin::engine::{
    ui::{UIManager, UIEvent, components::*, theme_engine::ThemeEngine},
    assets::{AssetDatabase, AssetPipeline, HotReloadSystem},
    core::Engine,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;

/// Integration Test Suite for Phase 3 Systems
///
/// This test suite validates the integration between:
/// - UI components and asset pipeline
/// - Theme engine integration across systems
/// - Hot reload with UI updates
/// - Database queries with UI search components
/// - Cross-system event handling
/// - Performance characteristics of integrated systems

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// Integration test fixture with all Phase 3 systems
    struct IntegrationTestFixture {
        engine: Engine,
        ui_manager: UIManager,
        asset_pipeline: AssetPipeline,
        asset_database: AssetDatabase,
        hot_reload: HotReloadSystem,
        theme_engine: ThemeEngine,
        test_assets_path: std::path::PathBuf,
    }

    impl IntegrationTestFixture {
        fn new() -> Self {
            let test_assets_path = std::path::PathBuf::from("tests/integration_assets");
            std::fs::create_dir_all(&test_assets_path).expect("Failed to create integration test assets directory");

            let asset_database = AssetDatabase::new("integration_test.db").expect("Failed to create integration database");
            let asset_pipeline = AssetPipeline::new(asset_database.clone());
            let hot_reload = HotReloadSystem::new(&test_assets_path);
            let mut ui_manager = UIManager::new();
            let theme_engine = ThemeEngine::new();
            let engine = Engine::new();

            // Connect systems
            ui_manager.set_asset_pipeline(Arc::clone(&asset_pipeline));
            ui_manager.set_theme_engine(Arc::clone(&theme_engine));

            Self {
                engine,
                ui_manager,
                asset_pipeline,
                asset_database,
                hot_reload,
                theme_engine,
                test_assets_path,
            }
        }

        fn create_test_ui_with_assets(&mut self) -> HashMap<String, ElementId> {
            let mut element_ids = HashMap::new();

            // Create UI components that use assets
            let button_with_icon = Button::new("Save Project".to_string())
                .with_icon_asset("icons/save.png")
                .with_variant(ButtonVariant::Primary);
            element_ids.insert("save_button".to_string(),
                self.ui_manager.add_element("save_button", Box::new(button_with_icon)));

            let modal_with_background = Modal::new("settings_modal".to_string(), "Settings".to_string())
                .with_background_image_asset("ui/modal_bg.png")
                .with_blur_background(true);
            element_ids.insert("settings_modal".to_string(),
                self.ui_manager.add_element("settings_modal", Box::new(modal_with_background)));

            let asset_browser = AssetBrowser::new("asset_browser".to_string())
                .with_thumbnail_size(128)
                .with_supported_formats(vec!["png", "jpg", "gltf", "fbx", "wav", "ogg"])
                .with_search_functionality(true);
            element_ids.insert("asset_browser".to_string(),
                self.ui_manager.add_element("asset_browser", Box::new(asset_browser)));

            element_ids
        }

        fn create_test_assets(&self) -> HashMap<String, std::path::PathBuf> {
            let mut asset_paths = HashMap::new();

            // Create test icon
            let icon_path = self.test_assets_path.join("icons").join("save.png");
            std::fs::create_dir_all(icon_path.parent().unwrap()).unwrap();
            self.create_test_icon(&icon_path);
            asset_paths.insert("save_icon".to_string(), icon_path);

            // Create test background
            let bg_path = self.test_assets_path.join("ui").join("modal_bg.png");
            std::fs::create_dir_all(bg_path.parent().unwrap()).unwrap();
            self.create_test_background(&bg_path);
            asset_paths.insert("modal_bg".to_string(), bg_path);

            // Create test thumbnails
            for i in 0..5 {
                let thumb_path = self.test_assets_path.join(format!("thumbnail_{}.png", i));
                self.create_test_thumbnail(&thumb_path, i);
                asset_paths.insert(format!("thumbnail_{}", i), thumb_path);
            }

            asset_paths
        }

        fn create_test_icon(&self, path: &std::path::Path) {
            // Create 32x32 icon PNG
            let icon_data = self.generate_icon_png_data(32, 32);
            std::fs::write(path, icon_data).expect("Failed to write test icon");
        }

        fn create_test_background(&self, path: &std::path::Path) {
            // Create 512x256 background PNG
            let bg_data = self.generate_background_png_data(512, 256);
            std::fs::write(path, bg_data).expect("Failed to write test background");
        }

        fn create_test_thumbnail(&self, path: &std::path::Path, index: usize) {
            // Create 128x128 thumbnail PNG with different patterns
            let thumb_data = self.generate_thumbnail_png_data(128, 128, index);
            std::fs::write(path, thumb_data).expect("Failed to write test thumbnail");
        }

        fn generate_icon_png_data(&self, width: u32, height: u32) -> Vec<u8> {
            // Minimal PNG data for testing
            let mut data = vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            ];

            // IHDR chunk
            data.extend_from_slice(&(13u32).to_be_bytes()); // length
            data.extend_from_slice(b"IHDR");
            data.extend_from_slice(&width.to_be_bytes());
            data.extend_from_slice(&height.to_be_bytes());
            data.extend_from_slice(&[8, 6, 0, 0, 0]); // bit depth, color type, compression, filter, interlace
            data.extend_from_slice(&[0x72, 0xB6, 0x0D, 0x24]); // CRC

            // IDAT chunk (minimal compressed data)
            let idat_data = vec![0x78, 0x9C, 0x63, 0x60, 0x60, 0x60, 0xF8, 0x0F, 0x00, 0x01, 0x01, 0x01, 0x00];
            data.extend_from_slice(&(idat_data.len() as u32).to_be_bytes());
            data.extend_from_slice(b"IDAT");
            data.extend_from_slice(&idat_data);
            data.extend_from_slice(&[0x18, 0xDD, 0x8D, 0xB4]); // CRC

            // IEND chunk
            data.extend_from_slice(&[0, 0, 0, 0]);
            data.extend_from_slice(b"IEND");
            data.extend_from_slice(&[0xAE, 0x42, 0x60, 0x82]);

            data
        }

        fn generate_background_png_data(&self, width: u32, height: u32) -> Vec<u8> {
            // Similar to icon but larger - reuse same structure
            self.generate_icon_png_data(width, height)
        }

        fn generate_thumbnail_png_data(&self, width: u32, height: u32, _pattern: usize) -> Vec<u8> {
            // Similar structure with different pattern data
            self.generate_icon_png_data(width, height)
        }
    }

    impl Drop for IntegrationTestFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.test_assets_path);
        }
    }

    #[test]
    fn test_ui_asset_integration() {
        let mut fixture = IntegrationTestFixture::new();
        let asset_paths = fixture.create_test_assets();
        let element_ids = fixture.create_test_ui_with_assets();

        // Import assets into the pipeline
        let mut asset_ids = HashMap::new();
        for (name, path) in asset_paths {
            let asset_id = fixture.asset_pipeline.import_asset(&path).expect("Failed to import asset");
            asset_ids.insert(name, asset_id);
        }

        // Test that UI components can access imported assets
        let save_button_id = element_ids["save_button"];
        let button_element = fixture.ui_manager.get_element(save_button_id).expect("Button should exist");

        // Verify button has icon asset reference
        assert!(button_element.has_icon_asset(), "Button should have icon asset");
        let icon_asset_id = button_element.get_icon_asset_id().expect("Button should have icon asset ID");

        // Verify asset exists in database
        let icon_asset = fixture.asset_database.get_asset(icon_asset_id).expect("Icon asset should exist in database");
        assert_eq!(icon_asset.asset_type, AssetType::Texture);
        assert!(icon_asset.width > 0 && icon_asset.height > 0);

        // Test modal background asset integration
        let modal_id = element_ids["settings_modal"];
        let modal_element = fixture.ui_manager.get_element(modal_id).expect("Modal should exist");

        assert!(modal_element.has_background_asset(), "Modal should have background asset");
        let bg_asset_id = modal_element.get_background_asset_id().expect("Modal should have background asset ID");

        let bg_asset = fixture.asset_database.get_asset(bg_asset_id).expect("Background asset should exist");
        assert_eq!(bg_asset.asset_type, AssetType::Texture);

        // Test asset browser integration
        let browser_id = element_ids["asset_browser"];
        let browser_element = fixture.ui_manager.get_element(browser_id).expect("Asset browser should exist");

        // Populate browser with database assets
        browser_element.refresh_from_database(&fixture.asset_database);
        let displayed_assets = browser_element.get_displayed_assets();

        assert!(!displayed_assets.is_empty(), "Asset browser should display imported assets");
        assert!(displayed_assets.len() >= asset_ids.len(), "Should display at least imported assets");
    }

    #[test]
    fn test_theme_engine_cross_system_integration() {
        let mut fixture = IntegrationTestFixture::new();
        let element_ids = fixture.create_test_ui_with_assets();

        // Test light theme application
        fixture.theme_engine.set_theme(Theme::light_mode());
        fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);

        // Verify theme applied to UI components
        for (name, element_id) in &element_ids {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");
            let computed_style = element.get_computed_style();

            assert!(computed_style.background_color.is_light(), "Light theme should use light backgrounds for {}", name);
            assert!(computed_style.text_color.is_dark(), "Light theme should use dark text for {}", name);

            // Check color contrast meets accessibility standards
            let contrast_ratio = fixture.theme_engine.calculate_contrast_ratio(
                &computed_style.background_color,
                &computed_style.text_color
            );
            assert!(contrast_ratio >= 4.5, "Contrast ratio should meet WCAG AA standards for {}", name);
        }

        // Test dark theme application
        fixture.theme_engine.set_theme(Theme::dark_mode());
        fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);

        for (name, element_id) in &element_ids {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");
            let computed_style = element.get_computed_style();

            assert!(computed_style.background_color.is_dark(), "Dark theme should use dark backgrounds for {}", name);
            assert!(computed_style.text_color.is_light(), "Dark theme should use light text for {}", name);
        }

        // Test high contrast theme
        fixture.theme_engine.set_theme(Theme::high_contrast_mode());
        fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);

        for (name, element_id) in &element_ids {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");
            let computed_style = element.get_computed_style();

            let contrast_ratio = fixture.theme_engine.calculate_contrast_ratio(
                &computed_style.background_color,
                &computed_style.text_color
            );
            assert!(contrast_ratio >= 7.0, "High contrast theme should exceed AAA standards for {}", name);
        }

        // Test custom theme integration with asset pipeline
        let mut custom_theme = Theme::new();
        custom_theme.set_background_texture_asset("ui/custom_bg.png");
        custom_theme.set_button_texture_asset("ui/custom_button.png");

        // Import theme assets
        let bg_path = fixture.test_assets_path.join("ui").join("custom_bg.png");
        let button_path = fixture.test_assets_path.join("ui").join("custom_button.png");

        std::fs::create_dir_all(bg_path.parent().unwrap()).unwrap();
        fixture.create_test_background(&bg_path);
        fixture.create_test_background(&button_path);

        let bg_asset_id = fixture.asset_pipeline.import_asset(&bg_path).expect("Failed to import background");
        let button_asset_id = fixture.asset_pipeline.import_asset(&button_path).expect("Failed to import button texture");

        custom_theme.link_background_asset(bg_asset_id);
        custom_theme.link_button_asset(button_asset_id);

        fixture.theme_engine.set_theme(custom_theme);
        fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);

        // Verify custom assets are used in theme
        let button_element = fixture.ui_manager.get_element(element_ids["save_button"]).expect("Button should exist");
        assert!(button_element.uses_custom_texture(), "Button should use custom theme texture");
    }

    #[test]
    fn test_hot_reload_ui_integration() {
        let mut fixture = IntegrationTestFixture::new();
        let asset_paths = fixture.create_test_assets();
        let element_ids = fixture.create_test_ui_with_assets();

        // Import initial assets
        let save_icon_path = &asset_paths["save_icon"];
        let icon_asset_id = fixture.asset_pipeline.import_asset(save_icon_path).expect("Failed to import icon");

        // Start hot reload monitoring
        fixture.hot_reload.start_monitoring();

        // Track UI update notifications
        let mut ui_updates_received = 0;
        fixture.ui_manager.set_asset_update_callback(Box::new(move |_asset_id| {
            ui_updates_received += 1;
        }));

        // Connect hot reload to UI system
        fixture.hot_reload.set_change_callback(Box::new({
            let ui_manager = Arc::clone(&fixture.ui_manager);
            let asset_pipeline = Arc::clone(&fixture.asset_pipeline);

            move |changed_files| {
                for file_path in changed_files {
                    if let Ok(asset_id) = asset_pipeline.reimport_asset(&file_path) {
                        ui_manager.notify_asset_updated(asset_id);
                    }
                }
            }
        }));

        // Simulate file change
        std::thread::sleep(Duration::from_millis(100));
        fixture.create_test_icon(save_icon_path); // Recreate with different timestamp

        // Wait for hot reload detection
        std::thread::sleep(Duration::from_millis(500));

        // Verify UI was notified of asset change
        assert!(ui_updates_received > 0, "UI should be notified of asset updates");

        // Verify UI elements using the asset were updated
        let button_element = fixture.ui_manager.get_element(element_ids["save_button"]).expect("Button should exist");
        assert!(button_element.is_asset_up_to_date(icon_asset_id), "Button should have updated asset");

        // Test hot reload with theme assets
        let theme_bg_path = fixture.test_assets_path.join("theme_bg.png");
        fixture.create_test_background(&theme_bg_path);
        let theme_asset_id = fixture.asset_pipeline.import_asset(&theme_bg_path).expect("Failed to import theme asset");

        let mut custom_theme = Theme::new();
        custom_theme.link_background_asset(theme_asset_id);
        fixture.theme_engine.set_theme(custom_theme);

        // Modify theme asset
        std::thread::sleep(Duration::from_millis(100));
        fixture.create_test_background(&theme_bg_path);
        std::thread::sleep(Duration::from_millis(500));

        // Verify theme engine was updated
        assert!(fixture.theme_engine.has_updated_assets(), "Theme engine should detect asset updates");

        // Verify all UI elements were re-themed
        for element_id in element_ids.values() {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");
            assert!(element.has_updated_theme(), "Element should have updated theme");
        }
    }

    #[test]
    fn test_database_search_ui_integration() {
        let mut fixture = IntegrationTestFixture::new();

        // Create diverse test assets with metadata
        let mut test_assets = Vec::new();
        let asset_types = vec![
            ("character_texture", "textures/character.png", AssetType::Texture),
            ("environment_model", "models/environment.gltf", AssetType::Model),
            ("ui_sound", "audio/ui_click.wav", AssetType::Audio),
            ("particle_texture", "effects/particle.png", AssetType::Texture),
            ("weapon_model", "models/weapon.fbx", AssetType::Model),
        ];

        for (name, path, asset_type) in asset_types {
            let full_path = fixture.test_assets_path.join(path);
            std::fs::create_dir_all(full_path.parent().unwrap()).unwrap();

            match asset_type {
                AssetType::Texture => fixture.create_test_icon(&full_path),
                AssetType::Model => fixture.create_test_background(&full_path),
                AssetType::Audio => {
                    std::fs::write(&full_path, b"fake audio data").unwrap();
                }
            }

            let asset_id = fixture.asset_pipeline.import_asset(&full_path).expect("Failed to import test asset");

            // Add metadata
            fixture.asset_database.set_asset_metadata(asset_id, "name", name);
            fixture.asset_database.set_asset_metadata(asset_id, "category",
                if name.contains("character") || name.contains("weapon") { "character" }
                else if name.contains("environment") { "environment" }
                else { "ui" }
            );
            fixture.asset_database.set_asset_metadata(asset_id, "size",
                if name.contains("texture") { "small" } else { "large" }
            );

            test_assets.push((name, asset_id, asset_type));
        }

        // Create search UI component
        let search_component = SearchComponent::new("asset_search".to_string())
            .with_database_connection(&fixture.asset_database)
            .with_real_time_results(true)
            .with_filters(vec!["type", "category", "size"])
            .with_sorting(vec!["name", "date_created", "file_size"]);

        let search_id = fixture.ui_manager.add_element("asset_search", Box::new(search_component));

        // Test basic text search
        let search_element = fixture.ui_manager.get_element_mut(search_id).expect("Search element should exist");
        search_element.set_search_query("character");

        let search_results = search_element.get_search_results();
        assert_eq!(search_results.len(), 2, "Should find 2 character-related assets");

        let result_names: Vec<_> = search_results.iter()
            .map(|&asset_id| fixture.asset_database.get_asset_metadata(asset_id, "name").unwrap())
            .collect();
        assert!(result_names.contains(&"character_texture".to_string()));

        // Test filtered search
        search_element.add_filter("type", "Texture");
        let filtered_results = search_element.get_search_results();

        for &asset_id in &filtered_results {
            let asset = fixture.asset_database.get_asset(asset_id).expect("Asset should exist");
            assert_eq!(asset.asset_type, AssetType::Texture);
        }

        // Test category filter
        search_element.clear_filters();
        search_element.add_filter("category", "environment");
        let category_results = search_element.get_search_results();
        assert_eq!(category_results.len(), 1, "Should find 1 environment asset");

        // Test sorting
        search_element.clear_filters();
        search_element.set_search_query(""); // Show all
        search_element.set_sort_order("name", SortDirection::Ascending);

        let sorted_results = search_element.get_search_results();
        let sorted_names: Vec<_> = sorted_results.iter()
            .map(|&asset_id| fixture.asset_database.get_asset_metadata(asset_id, "name").unwrap())
            .collect();

        let mut expected_names = sorted_names.clone();
        expected_names.sort();
        assert_eq!(sorted_names, expected_names, "Results should be sorted alphabetically");

        // Test real-time search updates
        let mut update_count = 0;
        search_element.set_results_update_callback(Box::new(move |_results| {
            update_count += 1;
        }));

        // Simulate typing
        search_element.simulate_typing("tex", 100); // 100ms between characters
        std::thread::sleep(Duration::from_millis(500));

        assert!(update_count > 0, "Should receive real-time search updates");

        // Test search performance with larger dataset
        let start_time = Instant::now();
        search_element.set_search_query("model");
        let performance_results = search_element.get_search_results();
        let search_duration = start_time.elapsed();

        assert!(search_duration.as_millis() < 100, "Search should be fast even with database queries");
        assert!(!performance_results.is_empty(), "Should find model assets");
    }

    #[test]
    fn test_cross_system_event_handling() {
        let mut fixture = IntegrationTestFixture::new();
        let asset_paths = fixture.create_test_assets();
        let element_ids = fixture.create_test_ui_with_assets();

        // Set up cross-system event tracking
        let mut ui_events_received = Vec::new();
        let mut asset_events_received = Vec::new();
        let mut theme_events_received = Vec::new();

        // Connect event systems
        fixture.ui_manager.set_global_event_handler(Box::new({
            let mut ui_events = ui_events_received.clone();
            move |event| {
                ui_events.push(format!("UI: {:?}", event));
            }
        }));

        fixture.asset_pipeline.set_event_handler(Box::new({
            let mut asset_events = asset_events_received.clone();
            move |event| {
                asset_events.push(format!("Asset: {:?}", event));
            }
        }));

        fixture.theme_engine.set_event_handler(Box::new({
            let mut theme_events = theme_events_received.clone();
            move |event| {
                theme_events.push(format!("Theme: {:?}", event));
            }
        }));

        // Test UI action triggering asset operations
        let save_button_id = element_ids["save_button"];
        let click_event = UIEvent::Click {
            element_id: save_button_id,
            position: (100.0, 50.0),
        };

        // Set up button to trigger asset save
        let button_element = fixture.ui_manager.get_element_mut(save_button_id).expect("Button should exist");
        button_element.set_click_handler(Box::new({
            let asset_pipeline = Arc::clone(&fixture.asset_pipeline);
            move || {
                // Simulate saving project assets
                asset_pipeline.save_all_assets();
            }
        }));

        fixture.ui_manager.handle_event(&click_event);

        // Verify cross-system event propagation
        std::thread::sleep(Duration::from_millis(100));
        assert!(!ui_events_received.is_empty(), "UI events should be received");
        assert!(!asset_events_received.is_empty(), "Asset events should be received from UI action");

        // Test asset change triggering UI updates
        let icon_path = &asset_paths["save_icon"];
        fixture.create_test_icon(icon_path); // Modify asset

        let asset_id = fixture.asset_pipeline.reimport_asset(icon_path).expect("Failed to reimport asset");

        std::thread::sleep(Duration::from_millis(100));

        // Verify UI received asset update notification
        let ui_event_contains_asset_update = ui_events_received.iter()
            .any(|event| event.contains("AssetUpdated"));
        assert!(ui_event_contains_asset_update, "UI should receive asset update events");

        // Test theme change triggering cross-system updates
        fixture.theme_engine.set_theme(Theme::dark_mode());

        std::thread::sleep(Duration::from_millis(100));

        assert!(!theme_events_received.is_empty(), "Theme events should be received");

        let ui_event_contains_theme_change = ui_events_received.iter()
            .any(|event| event.contains("ThemeChanged"));
        assert!(ui_event_contains_theme_change, "UI should receive theme change events");

        // Test error propagation across systems
        let invalid_asset_path = fixture.test_assets_path.join("invalid.xyz");
        std::fs::write(&invalid_asset_path, b"invalid").unwrap();

        let import_result = fixture.asset_pipeline.import_asset(&invalid_asset_path);
        assert!(import_result.is_err(), "Should fail to import invalid asset");

        std::thread::sleep(Duration::from_millis(100));

        let asset_error_event = asset_events_received.iter()
            .any(|event| event.contains("ImportError"));
        assert!(asset_error_event, "Asset error events should be propagated");

        let ui_error_notification = ui_events_received.iter()
            .any(|event| event.contains("ErrorNotification"));
        assert!(ui_error_notification, "UI should receive error notifications from asset system");
    }

    #[test]
    fn test_integrated_performance_characteristics() {
        let mut fixture = IntegrationTestFixture::new();

        // Create a complex UI with many asset-dependent components
        let component_count = 100;
        let mut element_ids = Vec::new();

        for i in 0..component_count {
            // Create asset for each component
            let asset_path = fixture.test_assets_path.join(format!("perf_asset_{}.png", i));
            fixture.create_test_icon(&asset_path);
            let asset_id = fixture.asset_pipeline.import_asset(&asset_path).expect("Failed to import performance test asset");

            // Create UI component using the asset
            let button = Button::new(format!("Button {}", i))
                .with_icon_asset_id(asset_id)
                .with_variant(ButtonVariant::Primary);

            let element_id = fixture.ui_manager.add_element(&format!("perf_button_{}", i), Box::new(button));
            element_ids.push(element_id);
        }

        // Test rendering performance with all components and assets
        let start_time = Instant::now();
        fixture.ui_manager.render_all_components();
        let render_duration = start_time.elapsed();

        assert!(render_duration.as_millis() < 50,
            "Rendering {} components with assets should be fast: {}ms",
            component_count, render_duration.as_millis());

        // Test theme switching performance with many components
        let start_time = Instant::now();
        fixture.theme_engine.set_theme(Theme::dark_mode());
        fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);
        let theme_switch_duration = start_time.elapsed();

        assert!(theme_switch_duration.as_millis() < 100,
            "Theme switching with {} components should be fast: {}ms",
            component_count, theme_switch_duration.as_millis());

        // Test asset search performance with UI integration
        let search_component = SearchComponent::new("perf_search".to_string())
            .with_database_connection(&fixture.asset_database)
            .with_real_time_results(true);

        let search_id = fixture.ui_manager.add_element("perf_search", Box::new(search_component));

        let start_time = Instant::now();
        let search_element = fixture.ui_manager.get_element_mut(search_id).expect("Search element should exist");
        search_element.set_search_query("perf");
        let search_results = search_element.get_search_results();
        let search_duration = start_time.elapsed();

        assert!(search_duration.as_millis() < 50,
            "Database search with UI integration should be fast: {}ms",
            search_duration.as_millis());
        assert!(search_results.len() >= component_count,
            "Should find all performance test assets");

        // Test memory efficiency of integrated systems
        let initial_memory = fixture.engine.get_memory_usage();

        // Force garbage collection if available
        fixture.ui_manager.cleanup_unused_elements();
        fixture.asset_pipeline.cleanup_unused_assets();

        let final_memory = fixture.engine.get_memory_usage();
        let memory_per_component = (final_memory - initial_memory) / component_count as u64;

        assert!(memory_per_component < 2048,
            "Memory usage per integrated component should be reasonable: {} bytes",
            memory_per_component);

        // Test concurrent operations performance
        let start_time = Instant::now();

        let handles: Vec<_> = (0..10).map(|i| {
            let ui_manager = Arc::clone(&fixture.ui_manager);
            let theme_engine = Arc::clone(&fixture.theme_engine);
            let asset_pipeline = Arc::clone(&fixture.asset_pipeline);

            std::thread::spawn(move || {
                match i % 3 {
                    0 => {
                        // UI operations
                        for j in 0..10 {
                            let button = Button::new(format!("Concurrent Button {}-{}", i, j));
                            ui_manager.add_element(&format!("concurrent_{}_{}", i, j), Box::new(button));
                        }
                    }
                    1 => {
                        // Theme operations
                        for _ in 0..10 {
                            theme_engine.compute_theme_variations();
                        }
                    }
                    2 => {
                        // Asset operations
                        for j in 0..10 {
                            asset_pipeline.get_asset_statistics();
                        }
                    }
                    _ => unreachable!()
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        let concurrent_duration = start_time.elapsed();
        assert!(concurrent_duration.as_millis() < 1000,
            "Concurrent operations across systems should complete quickly: {}ms",
            concurrent_duration.as_millis());
    }

    #[test]
    fn test_system_state_consistency() {
        let mut fixture = IntegrationTestFixture::new();
        let asset_paths = fixture.create_test_assets();
        let element_ids = fixture.create_test_ui_with_assets();

        // Import assets and verify initial state consistency
        let mut asset_ids = HashMap::new();
        for (name, path) in &asset_paths {
            let asset_id = fixture.asset_pipeline.import_asset(path).expect("Failed to import asset");
            asset_ids.insert(name.clone(), asset_id);
        }

        // Verify UI elements correctly reference imported assets
        for (name, element_id) in &element_ids {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");

            if element.has_asset_dependencies() {
                let dependencies = element.get_asset_dependencies();
                for dep_asset_id in dependencies {
                    assert!(fixture.asset_database.asset_exists(dep_asset_id),
                        "Element {} references non-existent asset", name);

                    let asset = fixture.asset_database.get_asset(dep_asset_id).expect("Asset should exist");
                    assert!(asset.is_loaded(), "Referenced asset should be loaded");
                }
            }
        }

        // Test state consistency during theme changes
        let original_theme = fixture.theme_engine.get_current_theme().clone();
        fixture.theme_engine.set_theme(Theme::dark_mode());

        // Verify all UI elements have consistent theme state
        for element_id in element_ids.values() {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");
            let element_theme = element.get_current_theme();
            assert_eq!(element_theme.theme_id(), fixture.theme_engine.get_current_theme().theme_id(),
                "Element theme should match global theme");
        }

        // Test state consistency during asset updates
        let save_icon_path = &asset_paths["save_icon"];
        std::thread::sleep(Duration::from_millis(100));
        fixture.create_test_icon(save_icon_path); // Update asset

        let updated_asset_id = fixture.asset_pipeline.reimport_asset(save_icon_path).expect("Failed to reimport asset");

        // Verify all UI elements using this asset are updated
        for element_id in element_ids.values() {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");

            if element.uses_asset(updated_asset_id) {
                assert!(element.is_asset_up_to_date(updated_asset_id),
                    "Element should have updated asset reference");
            }
        }

        // Test state consistency during asset removal
        let test_asset_path = fixture.test_assets_path.join("removable.png");
        fixture.create_test_icon(&test_asset_path);
        let removable_asset_id = fixture.asset_pipeline.import_asset(&test_asset_path).expect("Failed to import removable asset");

        // Create UI element using the removable asset
        let temp_button = Button::new("Temporary".to_string())
            .with_icon_asset_id(removable_asset_id);
        let temp_button_id = fixture.ui_manager.add_element("temp_button", Box::new(temp_button));

        // Remove the asset
        fixture.asset_pipeline.remove_asset(removable_asset_id);

        // Verify UI element handles missing asset gracefully
        let temp_element = fixture.ui_manager.get_element(temp_button_id).expect("Element should still exist");
        assert!(temp_element.has_missing_assets(), "Element should detect missing assets");
        assert!(temp_element.can_render_without_assets(), "Element should handle missing assets gracefully");

        // Test rollback consistency
        fixture.theme_engine.set_theme(original_theme);

        for element_id in element_ids.values() {
            let element = fixture.ui_manager.get_element(*element_id).expect("Element should exist");
            let element_theme = element.get_current_theme();
            assert_eq!(element_theme.theme_id(), fixture.theme_engine.get_current_theme().theme_id(),
                "Element theme should revert to original");
        }
    }
}

/// Mock implementations for integration testing
#[cfg(test)]
mod integration_mocks {
    use super::*;

    pub struct AssetBrowser {
        id: String,
        thumbnail_size: u32,
        supported_formats: Vec<String>,
        search_enabled: bool,
        displayed_assets: Vec<AssetId>,
    }

    impl AssetBrowser {
        pub fn new(id: String) -> Self {
            Self {
                id,
                thumbnail_size: 64,
                supported_formats: Vec::new(),
                search_enabled: false,
                displayed_assets: Vec::new(),
            }
        }

        pub fn with_thumbnail_size(mut self, size: u32) -> Self {
            self.thumbnail_size = size;
            self
        }

        pub fn with_supported_formats(mut self, formats: Vec<&str>) -> Self {
            self.supported_formats = formats.into_iter().map(|s| s.to_string()).collect();
            self
        }

        pub fn with_search_functionality(mut self, enabled: bool) -> Self {
            self.search_enabled = enabled;
            self
        }

        pub fn refresh_from_database(&mut self, _database: &AssetDatabase) {
            // Mock implementation
            self.displayed_assets = vec![AssetId::new(1), AssetId::new(2), AssetId::new(3)];
        }

        pub fn get_displayed_assets(&self) -> &[AssetId] {
            &self.displayed_assets
        }
    }

    pub struct SearchComponent {
        id: String,
        database: Option<Arc<AssetDatabase>>,
        real_time: bool,
        filters: Vec<String>,
        sorting: Vec<String>,
        current_query: String,
        current_results: Vec<AssetId>,
    }

    impl SearchComponent {
        pub fn new(id: String) -> Self {
            Self {
                id,
                database: None,
                real_time: false,
                filters: Vec::new(),
                sorting: Vec::new(),
                current_query: String::new(),
                current_results: Vec::new(),
            }
        }

        pub fn with_database_connection(mut self, database: &AssetDatabase) -> Self {
            self.database = Some(Arc::new(database.clone()));
            self
        }

        pub fn with_real_time_results(mut self, enabled: bool) -> Self {
            self.real_time = enabled;
            self
        }

        pub fn with_filters(mut self, filters: Vec<&str>) -> Self {
            self.filters = filters.into_iter().map(|s| s.to_string()).collect();
            self
        }

        pub fn with_sorting(mut self, sorting: Vec<&str>) -> Self {
            self.sorting = sorting.into_iter().map(|s| s.to_string()).collect();
            self
        }

        pub fn set_search_query(&mut self, query: &str) {
            self.current_query = query.to_string();
            self.update_results();
        }

        pub fn add_filter(&mut self, _filter_type: &str, _value: &str) {
            self.update_results();
        }

        pub fn clear_filters(&mut self) {
            self.update_results();
        }

        pub fn set_sort_order(&mut self, _field: &str, _direction: SortDirection) {
            self.update_results();
        }

        pub fn get_search_results(&self) -> &[AssetId] {
            &self.current_results
        }

        pub fn simulate_typing(&mut self, text: &str, _delay_ms: u64) {
            self.set_search_query(text);
        }

        pub fn set_results_update_callback(&mut self, _callback: Box<dyn Fn(&[AssetId])>) {
            // Mock implementation
        }

        fn update_results(&mut self) {
            // Mock search implementation
            self.current_results = if self.current_query.contains("character") {
                vec![AssetId::new(1), AssetId::new(2)]
            } else if self.current_query.contains("model") {
                vec![AssetId::new(3), AssetId::new(4)]
            } else if self.current_query.is_empty() {
                vec![AssetId::new(1), AssetId::new(2), AssetId::new(3), AssetId::new(4), AssetId::new(5)]
            } else {
                vec![AssetId::new(1)]
            };
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct AssetId(u32);

    impl AssetId {
        pub fn new(id: u32) -> Self {
            Self(id)
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum SortDirection {
        Ascending,
        Descending,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum AssetType {
        Texture,
        Model,
        Audio,
        Material,
    }
}