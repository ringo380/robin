use robin::engine::{
    ui::{UIManager, components::*},
    assets::{AssetPipeline, AssetDatabase},
    graphics::Renderer,
    core::Engine,
};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;
use criterion::{black_box, Criterion};

/// Performance Test Suite for Phase 3 Systems
///
/// This test suite validates performance characteristics of:
/// - UI component rendering and interaction
/// - Asset import/processing pipeline
/// - Memory usage optimization
/// - Hot reload response times
/// - Database query performance
/// - Theme switching performance
/// - Concurrent operations
/// - Large dataset handling

#[cfg(test)]
mod performance_tests {
    use super::*;

    struct PerformanceTestFixture {
        ui_manager: UIManager,
        asset_pipeline: AssetPipeline,
        asset_database: AssetDatabase,
        renderer: Renderer,
        engine: Engine,
        test_data_path: std::path::PathBuf,
    }

    impl PerformanceTestFixture {
        fn new() -> Self {
            let test_data_path = std::path::PathBuf::from("tests/performance_data");
            std::fs::create_dir_all(&test_data_path).expect("Failed to create performance test directory");

            let asset_database = AssetDatabase::new("performance_test.db").expect("Failed to create performance database");
            let asset_pipeline = AssetPipeline::new(asset_database.clone());
            let ui_manager = UIManager::new();
            let renderer = Renderer::new();
            let engine = Engine::new();

            Self {
                ui_manager,
                asset_pipeline,
                asset_database,
                renderer,
                engine,
                test_data_path,
            }
        }

        fn generate_test_assets(&self, count: usize) -> Vec<std::path::PathBuf> {
            let mut asset_paths = Vec::new();

            for i in 0..count {
                let asset_type = i % 4;
                let path = match asset_type {
                    0 => {
                        let path = self.test_data_path.join(format!("texture_{}.png", i));
                        self.create_test_texture(&path, 256, 256);
                        path
                    }
                    1 => {
                        let path = self.test_data_path.join(format!("model_{}.gltf", i));
                        self.create_test_model(&path, 1000); // 1000 vertices
                        path
                    }
                    2 => {
                        let path = self.test_data_path.join(format!("audio_{}.wav", i));
                        self.create_test_audio(&path, 44100, 2.0); // 44.1kHz, 2 seconds
                        path
                    }
                    3 => {
                        let path = self.test_data_path.join(format!("material_{}.json", i));
                        self.create_test_material(&path);
                        path
                    }
                    _ => unreachable!()
                };
                asset_paths.push(path);
            }

            asset_paths
        }

        fn create_test_texture(&self, path: &std::path::Path, width: u32, height: u32) {
            // Generate procedural texture data
            let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
            for y in 0..height {
                for x in 0..width {
                    let r = ((x as f32 / width as f32) * 255.0) as u8;
                    let g = ((y as f32 / height as f32) * 255.0) as u8;
                    let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
                    rgba_data.extend_from_slice(&[r, g, b, 255]);
                }
            }

            // Create minimal PNG structure (for testing purposes)
            let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
            std::fs::write(path, png_header).expect("Failed to write test texture");
        }

        fn create_test_model(&self, path: &std::path::Path, vertex_count: u32) {
            // Generate procedural model data
            let mut vertices = Vec::new();
            let mut indices = Vec::new();

            for i in 0..vertex_count {
                let angle = (i as f32 / vertex_count as f32) * 2.0 * std::f32::consts::PI;
                let radius = 1.0 + (i as f32 / vertex_count as f32) * 0.5;

                vertices.push(radius * angle.cos());
                vertices.push(radius * angle.sin());
                vertices.push((i as f32 / vertex_count as f32) * 2.0 - 1.0);

                if i >= 2 {
                    indices.extend_from_slice(&[0, i - 1, i]);
                }
            }

            let gltf_content = format!(r#"{{
                "scene": 0,
                "scenes": [{{"nodes": [0]}}],
                "nodes": [{{"mesh": 0}}],
                "meshes": [{{
                    "primitives": [{{
                        "attributes": {{"POSITION": 0}},
                        "indices": 1
                    }}]
                }}],
                "buffers": [{{"byteLength": {}}}],
                "asset": {{"version": "2.0"}}
            }}"#, vertices.len() * 4 + indices.len() * 2);

            std::fs::write(path, gltf_content).expect("Failed to write test model");
        }

        fn create_test_audio(&self, path: &std::path::Path, sample_rate: u32, duration: f32) {
            let sample_count = (sample_rate as f32 * duration) as usize;
            let mut samples = Vec::with_capacity(sample_count * 2); // Stereo

            for i in 0..sample_count {
                let t = i as f32 / sample_rate as f32;
                let frequency = 440.0; // A4
                let amplitude = 0.1;
                let sample = (amplitude * (2.0 * std::f32::consts::PI * frequency * t).sin() * 32767.0) as i16;
                samples.push(sample); // Left channel
                samples.push(sample); // Right channel
            }

            // Create minimal WAV structure
            let wav_header = b"RIFF\x24\x08\x00\x00WAVEfmt \x10\x00\x00\x00\x01\x00\x02\x00\x44\xAC\x00\x00\x10\xB1\x02\x00\x04\x00\x10\x00data\x00\x08\x00\x00";
            std::fs::write(path, wav_header).expect("Failed to write test audio");
        }

        fn create_test_material(&self, path: &std::path::Path) {
            let material_json = r#"{
                "name": "TestMaterial",
                "albedo": [0.8, 0.6, 0.4, 1.0],
                "metallic": 0.2,
                "roughness": 0.8,
                "normal_texture": null,
                "emission": [0.0, 0.0, 0.0]
            }"#;
            std::fs::write(path, material_json).expect("Failed to write test material");
        }

        fn create_ui_stress_test(&mut self, component_count: usize) -> Vec<ElementId> {
            let mut element_ids = Vec::new();

            for i in 0..component_count {
                let component_type = i % 6;
                let element_id = match component_type {
                    0 => {
                        let button = Button::new(format!("Button {}", i))
                            .with_variant(if i % 2 == 0 { ButtonVariant::Primary } else { ButtonVariant::Secondary })
                            .with_size(ButtonSize::Medium);
                        self.ui_manager.add_element(&format!("stress_button_{}", i), Box::new(button))
                    }
                    1 => {
                        let mut form = Form::new(format!("stress_form_{}", i));
                        form.add_text_field("field1", "Field 1", true, None);
                        form.add_email_field("field2", "Field 2", false, None);
                        self.ui_manager.add_element(&format!("stress_form_{}", i), Box::new(form))
                    }
                    2 => {
                        let modal = Modal::new(format!("stress_modal_{}", i), format!("Modal {}", i))
                            .with_size(ModalSize::Medium);
                        self.ui_manager.add_element(&format!("stress_modal_{}", i), Box::new(modal))
                    }
                    3 => {
                        let layout = Layout::flex(format!("stress_layout_{}", i))
                            .with_direction(if i % 2 == 0 { FlexDirection::Row } else { FlexDirection::Column });
                        self.ui_manager.add_element(&format!("stress_layout_{}", i), Box::new(layout))
                    }
                    4 => {
                        let mut nav = Navigation::new(format!("stress_nav_{}", i));
                        for j in 0..3 {
                            nav.add_item(&format!("item_{}", j), &format!("Item {}", j), &format!("/item{}", j), None);
                        }
                        self.ui_manager.add_element(&format!("stress_nav_{}", i), Box::new(nav))
                    }
                    5 => {
                        let feedback = Feedback::info(format!("Info message {}", i))
                            .with_auto_dismiss(5000);
                        self.ui_manager.add_element(&format!("stress_feedback_{}", i), Box::new(feedback))
                    }
                    _ => unreachable!()
                };
                element_ids.push(element_id);
            }

            element_ids
        }
    }

    impl Drop for PerformanceTestFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.test_data_path);
        }
    }

    #[test]
    fn test_ui_component_rendering_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Test rendering performance with various component counts
        let component_counts = vec![10, 50, 100, 500, 1000];

        for &count in &component_counts {
            let element_ids = fixture.create_ui_stress_test(count);

            // Measure rendering time
            let start_time = Instant::now();
            for _ in 0..10 {
                fixture.ui_manager.render_all_components();
            }
            let total_render_time = start_time.elapsed();
            let avg_render_time = total_render_time / 10;

            println!("Rendering {} components: {:.2}ms average", count, avg_render_time.as_secs_f64() * 1000.0);

            // Performance assertions
            match count {
                10 => assert!(avg_render_time.as_millis() < 1, "10 components should render in <1ms"),
                50 => assert!(avg_render_time.as_millis() < 5, "50 components should render in <5ms"),
                100 => assert!(avg_render_time.as_millis() < 10, "100 components should render in <10ms"),
                500 => assert!(avg_render_time.as_millis() < 25, "500 components should render in <25ms"),
                1000 => assert!(avg_render_time.as_millis() < 50, "1000 components should render in <50ms"),
                _ => {}
            }

            // Clean up components for next test
            for element_id in element_ids {
                fixture.ui_manager.remove_element(element_id);
            }
        }
    }

    #[test]
    fn test_ui_interaction_performance() {
        let mut fixture = PerformanceTestFixture::new();
        let element_ids = fixture.create_ui_stress_test(100);

        // Test click event handling performance
        let start_time = Instant::now();
        for (i, &element_id) in element_ids.iter().enumerate() {
            let click_event = UIEvent::Click {
                element_id,
                position: (100.0 + (i % 10) as f32 * 50.0, 50.0 + (i / 10) as f32 * 30.0),
            };
            fixture.ui_manager.handle_event(&click_event);
        }
        let interaction_time = start_time.elapsed();

        println!("Handling 100 click events: {:.2}ms", interaction_time.as_secs_f64() * 1000.0);
        assert!(interaction_time.as_millis() < 20, "100 click events should be handled in <20ms");

        // Test hover event performance
        let start_time = Instant::now();
        for (i, &element_id) in element_ids.iter().enumerate() {
            let hover_event = UIEvent::Hover {
                element_id,
                position: (100.0 + (i % 10) as f32 * 50.0, 50.0 + (i / 10) as f32 * 30.0),
                entered: true,
            };
            fixture.ui_manager.handle_event(&hover_event);
        }
        let hover_time = start_time.elapsed();

        println!("Handling 100 hover events: {:.2}ms", hover_time.as_secs_f64() * 1000.0);
        assert!(hover_time.as_millis() < 15, "100 hover events should be handled in <15ms");
    }

    #[test]
    fn test_asset_import_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Test import performance with different asset counts
        let asset_counts = vec![10, 50, 100];

        for &count in &asset_counts {
            let asset_paths = fixture.generate_test_assets(count);

            // Sequential import performance
            let start_time = Instant::now();
            let mut import_results = Vec::new();
            for path in &asset_paths {
                let result = fixture.asset_pipeline.import_asset(path);
                import_results.push(result);
            }
            let sequential_time = start_time.elapsed();

            let successful_imports = import_results.iter().filter(|r| r.is_ok()).count();
            let avg_import_time = sequential_time.as_millis() / count as u128;

            println!("Sequential import of {} assets: {:.2}ms total, {:.2}ms average",
                count, sequential_time.as_secs_f64() * 1000.0, avg_import_time);

            assert!(successful_imports >= (count * 8 / 10), "At least 80% of imports should succeed");

            match count {
                10 => assert!(avg_import_time < 50, "Average import time should be <50ms for small batches"),
                50 => assert!(avg_import_time < 100, "Average import time should be <100ms for medium batches"),
                100 => assert!(avg_import_time < 200, "Average import time should be <200ms for large batches"),
                _ => {}
            }

            // Concurrent import performance
            let start_time = Instant::now();
            let chunk_size = (count / 4).max(1);
            let handles: Vec<_> = asset_paths.chunks(chunk_size)
                .map(|chunk| {
                    let chunk_paths = chunk.to_vec();
                    let pipeline = fixture.asset_pipeline.clone();
                    thread::spawn(move || {
                        chunk_paths.into_iter()
                            .map(|path| pipeline.import_asset(&path))
                            .collect::<Vec<_>>()
                    })
                })
                .collect();

            let concurrent_results: Vec<_> = handles.into_iter()
                .map(|h| h.join().unwrap())
                .flatten()
                .collect();

            let concurrent_time = start_time.elapsed();

            println!("Concurrent import of {} assets: {:.2}ms total",
                count, concurrent_time.as_secs_f64() * 1000.0);

            // Concurrent should be faster than sequential for larger batches
            if count >= 50 {
                assert!(concurrent_time < sequential_time * 8 / 10,
                    "Concurrent import should be significantly faster for {} assets", count);
            }

            assert!(concurrent_results.iter().filter(|r| r.is_ok()).count() >= (count * 8 / 10),
                "Concurrent import should maintain success rate");
        }
    }

    #[test]
    fn test_asset_database_query_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Import test assets for database testing
        let asset_paths = fixture.generate_test_assets(500);
        let mut asset_ids = Vec::new();

        for (i, path) in asset_paths.iter().enumerate() {
            if let Ok(asset_id) = fixture.asset_pipeline.import_asset(path) {
                // Add metadata for testing
                fixture.asset_database.set_asset_metadata(asset_id, "name", &format!("Asset_{}", i));
                fixture.asset_database.set_asset_metadata(asset_id, "category",
                    match i % 4 {
                        0 => "textures",
                        1 => "models",
                        2 => "audio",
                        3 => "materials",
                        _ => unreachable!()
                    }
                );
                fixture.asset_database.set_asset_metadata(asset_id, "size",
                    if i % 3 == 0 { "large" } else if i % 3 == 1 { "medium" } else { "small" }
                );
                asset_ids.push(asset_id);
            }
        }

        // Test search query performance
        let search_queries = vec![
            "Asset_1", "Asset_5", "textures", "models", "large", "medium", "small"
        ];

        for query in search_queries {
            let start_time = Instant::now();
            let results = fixture.asset_database.search_assets(query);
            let search_time = start_time.elapsed();

            println!("Search query '{}': {} results in {:.2}ms",
                query, results.len(), search_time.as_secs_f64() * 1000.0);

            assert!(search_time.as_millis() < 50,
                "Search query '{}' should complete in <50ms", query);
            assert!(!results.is_empty(), "Search query '{}' should return results", query);
        }

        // Test filtered queries
        let filter_combinations = vec![
            vec![("category", "textures")],
            vec![("size", "large")],
            vec![("category", "models"), ("size", "medium")],
        ];

        for filters in filter_combinations {
            let start_time = Instant::now();
            let mut query_builder = fixture.asset_database.query_builder();
            for (key, value) in &filters {
                query_builder = query_builder.filter(key, value);
            }
            let results = query_builder.execute();
            let filter_time = start_time.elapsed();

            println!("Filtered query {:?}: {} results in {:.2}ms",
                filters, results.len(), filter_time.as_secs_f64() * 1000.0);

            assert!(filter_time.as_millis() < 30,
                "Filtered query should complete in <30ms");
        }

        // Test sorting performance
        let sort_fields = vec!["name", "date_created", "file_size"];

        for field in sort_fields {
            let start_time = Instant::now();
            let results = fixture.asset_database.query_builder()
                .sort_by(field, SortDirection::Ascending)
                .limit(100)
                .execute();
            let sort_time = start_time.elapsed();

            println!("Sorted query by '{}': {} results in {:.2}ms",
                field, results.len(), sort_time.as_secs_f64() * 1000.0);

            assert!(sort_time.as_millis() < 40,
                "Sorted query should complete in <40ms");
            assert_eq!(results.len(), 100.min(asset_ids.len()),
                "Should return requested number of results");
        }
    }

    #[test]
    fn test_hot_reload_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Create and import initial assets
        let asset_count = 50;
        let asset_paths = fixture.generate_test_assets(asset_count);
        let mut asset_ids = Vec::new();

        for path in &asset_paths {
            if let Ok(asset_id) = fixture.asset_pipeline.import_asset(path) {
                asset_ids.push(asset_id);
            }
        }

        // Set up hot reload monitoring
        let mut reload_times = Vec::new();
        fixture.hot_reload.set_change_callback(Box::new({
            let pipeline = fixture.asset_pipeline.clone();
            let reload_times = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
            let reload_times_clone = reload_times.clone();

            move |changed_files| {
                let start_time = Instant::now();
                for file_path in changed_files {
                    let _ = pipeline.reimport_asset(&file_path);
                }
                let reload_time = start_time.elapsed();
                reload_times_clone.lock().unwrap().push(reload_time);
            }
        }));

        fixture.hot_reload.start_monitoring();

        // Simulate file changes and measure reload performance
        for (i, path) in asset_paths.iter().enumerate().take(10) {
            thread::sleep(Duration::from_millis(100)); // Ensure different timestamps

            let start_time = Instant::now();

            // Modify the file
            match i % 4 {
                0 => fixture.create_test_texture(path, 256, 256),
                1 => fixture.create_test_model(path, 1000),
                2 => fixture.create_test_audio(path, 44100, 2.0),
                3 => fixture.create_test_material(path),
                _ => unreachable!()
            }

            // Wait for hot reload to detect and process the change
            thread::sleep(Duration::from_millis(200));

            let total_reload_time = start_time.elapsed();
            println!("Hot reload for file {}: {:.2}ms", i, total_reload_time.as_secs_f64() * 1000.0);

            assert!(total_reload_time.as_millis() < 500,
                "Hot reload should complete within 500ms");
        }

        // Test bulk file changes
        let start_time = Instant::now();
        for path in &asset_paths[10..20] {
            fixture.create_test_texture(path, 256, 256);
        }
        thread::sleep(Duration::from_millis(500)); // Wait for all changes to be processed

        let bulk_reload_time = start_time.elapsed();
        println!("Bulk hot reload (10 files): {:.2}ms", bulk_reload_time.as_secs_f64() * 1000.0);

        assert!(bulk_reload_time.as_millis() < 2000,
            "Bulk hot reload should complete within 2 seconds");
    }

    #[test]
    fn test_theme_switching_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Create UI components for theme testing
        let element_ids = fixture.create_ui_stress_test(200);

        // Test performance of different theme switches
        let themes = vec![
            Theme::light_mode(),
            Theme::dark_mode(),
            Theme::high_contrast_mode(),
        ];

        for (i, theme) in themes.iter().enumerate() {
            let start_time = Instant::now();
            fixture.theme_engine.set_theme(theme.clone());
            fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);
            let theme_switch_time = start_time.elapsed();

            println!("Theme switch {} (200 components): {:.2}ms",
                i, theme_switch_time.as_secs_f64() * 1000.0);

            assert!(theme_switch_time.as_millis() < 100,
                "Theme switching should complete in <100ms for 200 components");
        }

        // Test rapid theme switching (user clicking between themes quickly)
        let start_time = Instant::now();
        for _ in 0..10 {
            fixture.theme_engine.set_theme(Theme::light_mode());
            fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);
            fixture.theme_engine.set_theme(Theme::dark_mode());
            fixture.ui_manager.apply_theme_to_all_elements(&fixture.theme_engine);
        }
        let rapid_switch_time = start_time.elapsed();

        println!("Rapid theme switching (20 switches): {:.2}ms",
            rapid_switch_time.as_secs_f64() * 1000.0);

        assert!(rapid_switch_time.as_millis() < 500,
            "Rapid theme switching should handle gracefully");
    }

    #[test]
    fn test_memory_usage_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Baseline memory usage
        let initial_memory = fixture.engine.get_memory_usage();
        println!("Initial memory usage: {} bytes", initial_memory);

        // Test UI component memory usage
        let ui_element_ids = fixture.create_ui_stress_test(1000);
        let ui_memory = fixture.engine.get_memory_usage();
        let ui_memory_per_component = (ui_memory - initial_memory) / 1000;

        println!("Memory per UI component: {} bytes", ui_memory_per_component);
        assert!(ui_memory_per_component < 1024,
            "Each UI component should use less than 1KB");

        // Test asset memory usage
        let asset_paths = fixture.generate_test_assets(100);
        let mut asset_ids = Vec::new();
        for path in &asset_paths {
            if let Ok(asset_id) = fixture.asset_pipeline.import_asset(path) {
                asset_ids.push(asset_id);
            }
        }

        let asset_memory = fixture.engine.get_memory_usage();
        let asset_memory_per_item = (asset_memory - ui_memory) / asset_ids.len() as u64;

        println!("Memory per asset: {} bytes", asset_memory_per_item);
        assert!(asset_memory_per_item < 10240,
            "Each asset should use less than 10KB on average");

        // Test memory cleanup
        for element_id in ui_element_ids {
            fixture.ui_manager.remove_element(element_id);
        }
        for asset_id in asset_ids {
            fixture.asset_pipeline.unload_asset(asset_id);
        }

        // Force cleanup
        fixture.ui_manager.cleanup_unused_elements();
        fixture.asset_pipeline.cleanup_unused_assets();

        let cleanup_memory = fixture.engine.get_memory_usage();
        let memory_reclaimed = asset_memory - cleanup_memory;

        println!("Memory reclaimed after cleanup: {} bytes", memory_reclaimed);
        assert!(cleanup_memory <= initial_memory * 110 / 100,
            "Memory should return close to initial level after cleanup");
    }

    #[test]
    fn test_concurrent_operations_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Test concurrent UI operations
        let start_time = Instant::now();
        let ui_handles: Vec<_> = (0..4).map(|thread_id| {
            let ui_manager = fixture.ui_manager.clone();
            thread::spawn(move || {
                for i in 0..50 {
                    let button = Button::new(format!("Concurrent Button {}-{}", thread_id, i));
                    ui_manager.add_element(&format!("concurrent_{}_{}", thread_id, i), Box::new(button));
                }
            })
        }).collect();

        for handle in ui_handles {
            handle.join().unwrap();
        }
        let concurrent_ui_time = start_time.elapsed();

        println!("Concurrent UI operations (4 threads, 50 components each): {:.2}ms",
            concurrent_ui_time.as_secs_f64() * 1000.0);

        assert!(concurrent_ui_time.as_millis() < 200,
            "Concurrent UI operations should complete quickly");

        // Test concurrent asset operations
        let asset_paths = fixture.generate_test_assets(40);
        let start_time = Instant::now();

        let asset_handles: Vec<_> = asset_paths.chunks(10)
            .map(|chunk| {
                let chunk_paths = chunk.to_vec();
                let pipeline = fixture.asset_pipeline.clone();
                thread::spawn(move || {
                    chunk_paths.into_iter()
                        .map(|path| pipeline.import_asset(&path))
                        .collect::<Vec<_>>()
                })
            })
            .collect();

        let concurrent_results: Vec<_> = asset_handles.into_iter()
            .map(|h| h.join().unwrap())
            .flatten()
            .collect();

        let concurrent_asset_time = start_time.elapsed();

        println!("Concurrent asset operations (4 threads, 10 assets each): {:.2}ms",
            concurrent_asset_time.as_secs_f64() * 1000.0);

        assert!(concurrent_asset_time.as_millis() < 1000,
            "Concurrent asset operations should complete within 1 second");

        let successful_imports = concurrent_results.iter().filter(|r| r.is_ok()).count();
        assert!(successful_imports >= 32,
            "At least 80% of concurrent imports should succeed");

        // Test mixed concurrent operations (UI + Assets + Theme)
        let start_time = Instant::now();

        let mixed_handles: Vec<_> = (0..3).map(|operation_type| {
            match operation_type {
                0 => {
                    let ui_manager = fixture.ui_manager.clone();
                    thread::spawn(move || {
                        for i in 0..30 {
                            let form = Form::new(format!("mixed_form_{}", i));
                            ui_manager.add_element(&format!("mixed_form_{}", i), Box::new(form));
                        }
                    })
                }
                1 => {
                    let asset_pipeline = fixture.asset_pipeline.clone();
                    let test_data_path = fixture.test_data_path.clone();
                    thread::spawn(move || {
                        for i in 0..20 {
                            let path = test_data_path.join(format!("mixed_texture_{}.png", i));
                            // Create simple test file
                            std::fs::write(&path, b"fake texture data").unwrap();
                            let _ = asset_pipeline.import_asset(&path);
                        }
                    })
                }
                2 => {
                    let theme_engine = fixture.theme_engine.clone();
                    thread::spawn(move || {
                        for _ in 0..20 {
                            theme_engine.compute_theme_variations();
                            thread::sleep(Duration::from_millis(5));
                        }
                    })
                }
                _ => unreachable!()
            }
        }).collect();

        for handle in mixed_handles {
            handle.join().unwrap();
        }

        let mixed_operations_time = start_time.elapsed();

        println!("Mixed concurrent operations: {:.2}ms",
            mixed_operations_time.as_secs_f64() * 1000.0);

        assert!(mixed_operations_time.as_millis() < 1500,
            "Mixed concurrent operations should complete within 1.5 seconds");
    }

    #[test]
    fn test_large_dataset_performance() {
        let mut fixture = PerformanceTestFixture::new();

        // Test with large numbers of assets
        let large_asset_count = 1000;
        println!("Testing with {} assets...", large_asset_count);

        let asset_paths = fixture.generate_test_assets(large_asset_count);

        // Batch import performance
        let batch_size = 100;
        let mut total_import_time = Duration::new(0, 0);
        let mut successful_imports = 0;

        for (batch_index, chunk) in asset_paths.chunks(batch_size).enumerate() {
            let start_time = Instant::now();

            for path in chunk {
                if fixture.asset_pipeline.import_asset(path).is_ok() {
                    successful_imports += 1;
                }
            }

            let batch_time = start_time.elapsed();
            total_import_time += batch_time;

            println!("Batch {} ({} assets): {:.2}ms",
                batch_index + 1, chunk.len(), batch_time.as_secs_f64() * 1000.0);

            // Each batch should complete within reasonable time
            assert!(batch_time.as_millis() < 5000,
                "Batch import should complete within 5 seconds");
        }

        let avg_import_time = total_import_time.as_millis() / successful_imports as u128;
        println!("Total import time: {:.2}s, Average per asset: {:.2}ms",
            total_import_time.as_secs_f64(), avg_import_time);

        assert!(successful_imports >= (large_asset_count * 8 / 10),
            "At least 80% of large dataset imports should succeed");
        assert!(avg_import_time < 500,
            "Average import time should remain reasonable for large datasets");

        // Database performance with large dataset
        let start_time = Instant::now();
        let all_assets = fixture.asset_database.get_all_assets();
        let db_query_time = start_time.elapsed();

        println!("Querying all {} assets: {:.2}ms",
            all_assets.len(), db_query_time.as_secs_f64() * 1000.0);

        assert!(db_query_time.as_millis() < 1000,
            "Querying large dataset should complete within 1 second");

        // Search performance with large dataset
        let search_terms = vec!["texture", "model", "audio", "material"];
        for term in search_terms {
            let start_time = Instant::now();
            let search_results = fixture.asset_database.search_assets(term);
            let search_time = start_time.elapsed();

            println!("Search '{}' in large dataset: {} results in {:.2}ms",
                term, search_results.len(), search_time.as_secs_f64() * 1000.0);

            assert!(search_time.as_millis() < 200,
                "Search in large dataset should complete within 200ms");
        }

        // UI performance with large asset browser
        let asset_browser = AssetBrowser::new("large_browser".to_string())
            .with_thumbnail_size(64)
            .with_pagination(50); // Show 50 items per page

        let browser_id = fixture.ui_manager.add_element("large_browser", Box::new(asset_browser));

        let start_time = Instant::now();
        let browser_element = fixture.ui_manager.get_element_mut(browser_id).expect("Browser should exist");
        browser_element.load_asset_database(&fixture.asset_database);
        let browser_load_time = start_time.elapsed();

        println!("Loading large dataset into asset browser: {:.2}ms",
            browser_load_time.as_secs_f64() * 1000.0);

        assert!(browser_load_time.as_millis() < 500,
            "Asset browser should load large dataset quickly");

        // Rendering performance with pagination
        let start_time = Instant::now();
        browser_element.render_current_page();
        let page_render_time = start_time.elapsed();

        println!("Rendering asset browser page (50 items): {:.2}ms",
            page_render_time.as_secs_f64() * 1000.0);

        assert!(page_render_time.as_millis() < 100,
            "Asset browser page rendering should be fast");
    }
}

/// Mock implementations for performance testing
#[cfg(test)]
mod performance_mocks {
    use super::*;

    pub struct AssetBrowser {
        id: String,
        thumbnail_size: u32,
        pagination_size: usize,
        current_page: usize,
        total_assets: usize,
    }

    impl AssetBrowser {
        pub fn new(id: String) -> Self {
            Self {
                id,
                thumbnail_size: 64,
                pagination_size: 20,
                current_page: 0,
                total_assets: 0,
            }
        }

        pub fn with_thumbnail_size(mut self, size: u32) -> Self {
            self.thumbnail_size = size;
            self
        }

        pub fn with_pagination(mut self, page_size: usize) -> Self {
            self.pagination_size = page_size;
            self
        }

        pub fn load_asset_database(&mut self, database: &AssetDatabase) {
            // Mock implementation
            self.total_assets = database.get_asset_count();
        }

        pub fn render_current_page(&self) {
            // Mock rendering operation
            let start_index = self.current_page * self.pagination_size;
            let end_index = (start_index + self.pagination_size).min(self.total_assets);

            // Simulate rendering work
            for _ in start_index..end_index {
                black_box(self.thumbnail_size);
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ElementId {
        Id(u32),
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum UIEvent {
        Click { element_id: ElementId, position: (f32, f32) },
        Hover { element_id: ElementId, position: (f32, f32), entered: bool },
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum SortDirection {
        Ascending,
        Descending,
    }
}