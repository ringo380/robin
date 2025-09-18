use robin::engine::{
    ui::{UIManager, UIEvent, components::*},
    assets::{AssetPipeline, AssetDatabase, AssetError},
    error::{RobinError, ErrorContext, ErrorRecovery},
    core::Engine,
};
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Duration;

/// Error Handling and Edge Cases Test Suite for Phase 3 Systems
///
/// This test suite validates robust error handling for:
/// - Invalid file formats and corrupted assets
/// - Missing dependencies and broken asset references
/// - Database connection failures and data corruption
/// - UI component error states and recovery
/// - Memory pressure and resource exhaustion
/// - Network connectivity issues
/// - Concurrent access conflicts
/// - Malformed configuration files
/// - Security vulnerabilities and input validation

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    struct ErrorTestFixture {
        ui_manager: UIManager,
        asset_pipeline: AssetPipeline,
        asset_database: AssetDatabase,
        engine: Engine,
        test_data_path: PathBuf,
    }

    impl ErrorTestFixture {
        fn new() -> Self {
            let test_data_path = PathBuf::from("tests/error_test_data");
            std::fs::create_dir_all(&test_data_path).expect("Failed to create error test directory");

            let asset_database = AssetDatabase::new("error_test.db").expect("Failed to create error test database");
            let asset_pipeline = AssetPipeline::new(asset_database.clone());
            let ui_manager = UIManager::new();
            let engine = Engine::new();

            Self {
                ui_manager,
                asset_pipeline,
                asset_database,
                engine,
                test_data_path,
            }
        }

        fn create_corrupted_png(&self, path: &PathBuf) {
            // Create file with PNG signature but corrupted data
            let corrupted_data = vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // Valid PNG signature
                0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0x00, // Corrupted IHDR
                0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, // Random garbage
            ];
            std::fs::write(path, corrupted_data).expect("Failed to write corrupted PNG");
        }

        fn create_malformed_gltf(&self, path: &PathBuf) {
            // Create JSON with invalid GLTF structure
            let malformed_json = r#"{
                "scene": "invalid_scene_reference",
                "scenes": [{"nodes": ["not_a_number"]}],
                "meshes": [{
                    "primitives": [{
                        "attributes": {"POSITION": -1},
                        "indices": "invalid_accessor"
                    }]
                }],
                "asset": {"version": "1.0"}
            }"#;
            std::fs::write(path, malformed_json).expect("Failed to write malformed GLTF");
        }

        fn create_invalid_audio(&self, path: &PathBuf) {
            // Create file with WAV signature but invalid header
            let invalid_data = vec![
                0x52, 0x49, 0x46, 0x46, // RIFF signature
                0xFF, 0xFF, 0xFF, 0xFF, // Invalid chunk size
                0x57, 0x41, 0x56, 0x45, // WAVE format
                0x00, 0x00, 0x00, 0x00, // Invalid subchunk
            ];
            std::fs::write(path, invalid_data).expect("Failed to write invalid audio");
        }

        fn create_empty_file(&self, path: &PathBuf) {
            std::fs::write(path, b"").expect("Failed to write empty file");
        }

        fn create_extremely_large_file(&self, path: &PathBuf) -> std::io::Result<()> {
            // Create a file that's too large to reasonably process
            let large_data = vec![0u8; 100 * 1024 * 1024]; // 100MB of zeros
            std::fs::write(path, large_data)
        }

        fn simulate_database_corruption(&mut self) {
            // Simulate database corruption by writing invalid data to the database file
            if let Some(db_path) = self.asset_database.get_database_path() {
                std::fs::write(db_path, b"corrupted database content").ok();
            }
        }

        fn simulate_memory_pressure(&mut self) {
            // Simulate memory pressure by allocating large amounts of memory
            self.engine.set_memory_limit(1024 * 1024); // 1MB limit
        }

        fn create_circular_dependency_assets(&self) -> (PathBuf, PathBuf) {
            let asset_a_path = self.test_data_path.join("circular_a.gltf");
            let asset_b_path = self.test_data_path.join("circular_b.gltf");

            // Asset A references Asset B
            let gltf_a = format!(r#"{{
                "scene": 0,
                "scenes": [{{"nodes": [0]}}],
                "nodes": [{{"mesh": 0}}],
                "meshes": [{{
                    "primitives": [{{
                        "attributes": {{"POSITION": 0}},
                        "material": 0
                    }}]
                }}],
                "materials": [{{
                    "name": "MaterialA",
                    "pbrMetallicRoughness": {{
                        "baseColorTexture": {{
                            "index": 0,
                            "source": "{}"
                        }}
                    }}
                }}],
                "asset": {{"version": "2.0"}}
            }}"#, asset_b_path.to_string_lossy());

            // Asset B references Asset A
            let gltf_b = format!(r#"{{
                "scene": 0,
                "scenes": [{{"nodes": [0]}}],
                "nodes": [{{"mesh": 0}}],
                "meshes": [{{
                    "primitives": [{{
                        "attributes": {{"POSITION": 0}},
                        "material": 0
                    }}]
                }}],
                "materials": [{{
                    "name": "MaterialB",
                    "pbrMetallicRoughness": {{
                        "baseColorTexture": {{
                            "index": 0,
                            "source": "{}"
                        }}
                    }}
                }}],
                "asset": {{"version": "2.0"}}
            }}"#, asset_a_path.to_string_lossy());

            std::fs::write(&asset_a_path, gltf_a).expect("Failed to write circular asset A");
            std::fs::write(&asset_b_path, gltf_b).expect("Failed to write circular asset B");

            (asset_a_path, asset_b_path)
        }
    }

    impl Drop for ErrorTestFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.test_data_path);
        }
    }

    #[test]
    fn test_invalid_file_format_handling() {
        let mut fixture = ErrorTestFixture::new();

        // Test unknown file extension
        let unknown_file = fixture.test_data_path.join("unknown.xyz");
        std::fs::write(&unknown_file, b"unknown format content").expect("Failed to create unknown file");

        let result = fixture.asset_pipeline.import_asset(&unknown_file);
        assert!(result.is_err(), "Should reject unknown file format");
        assert!(matches!(result.unwrap_err(), AssetError::UnsupportedFormat(_)));

        // Test file with wrong extension (PNG data with .txt extension)
        let wrong_ext_file = fixture.test_data_path.join("fake.txt");
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG signature
        std::fs::write(&wrong_ext_file, png_data).expect("Failed to create wrong extension file");

        let result = fixture.asset_pipeline.import_asset(&wrong_ext_file);
        // Should either detect as PNG or reject based on extension
        if let Err(error) = result {
            assert!(matches!(error, AssetError::UnsupportedFormat(_) | AssetError::FormatMismatch(_)));
        }

        // Test completely invalid binary data
        let garbage_file = fixture.test_data_path.join("garbage.png");
        let garbage_data = (0..1024).map(|i| (i % 256) as u8).collect::<Vec<_>>();
        std::fs::write(&garbage_file, garbage_data).expect("Failed to create garbage file");

        let result = fixture.asset_pipeline.import_asset(&garbage_file);
        assert!(result.is_err(), "Should reject garbage data");
        assert!(matches!(result.unwrap_err(), AssetError::CorruptedFile(_) | AssetError::ParseError(_)));
    }

    #[test]
    fn test_corrupted_asset_handling() {
        let mut fixture = ErrorTestFixture::new();

        // Test corrupted PNG
        let corrupted_png = fixture.test_data_path.join("corrupted.png");
        fixture.create_corrupted_png(&corrupted_png);

        let result = fixture.asset_pipeline.import_asset(&corrupted_png);
        assert!(result.is_err(), "Should reject corrupted PNG");
        assert!(matches!(result.unwrap_err(), AssetError::CorruptedFile(_)));

        // Test malformed GLTF
        let malformed_gltf = fixture.test_data_path.join("malformed.gltf");
        fixture.create_malformed_gltf(&malformed_gltf);

        let result = fixture.asset_pipeline.import_asset(&malformed_gltf);
        assert!(result.is_err(), "Should reject malformed GLTF");
        assert!(matches!(result.unwrap_err(), AssetError::ParseError(_) | AssetError::InvalidStructure(_)));

        // Test invalid audio file
        let invalid_audio = fixture.test_data_path.join("invalid.wav");
        fixture.create_invalid_audio(&invalid_audio);

        let result = fixture.asset_pipeline.import_asset(&invalid_audio);
        assert!(result.is_err(), "Should reject invalid audio");
        assert!(matches!(result.unwrap_err(), AssetError::CorruptedFile(_) | AssetError::InvalidAudioFormat(_)));

        // Test empty files
        let empty_file = fixture.test_data_path.join("empty.png");
        fixture.create_empty_file(&empty_file);

        let result = fixture.asset_pipeline.import_asset(&empty_file);
        assert!(result.is_err(), "Should reject empty file");
        assert!(matches!(result.unwrap_err(), AssetError::EmptyFile(_)));

        // Verify error recovery - pipeline should continue working after errors
        let valid_file = fixture.test_data_path.join("valid_after_errors.png");
        let valid_png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            // Minimal valid PNG structure
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
            0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41,
            0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
            0x42, 0x60, 0x82,
        ];
        std::fs::write(&valid_file, valid_png_data).expect("Failed to create valid file");

        let result = fixture.asset_pipeline.import_asset(&valid_file);
        assert!(result.is_ok(), "Should successfully import valid file after errors");
    }

    #[test]
    fn test_missing_dependency_handling() {
        let mut fixture = ErrorTestFixture::new();

        // Create GLTF that references missing texture
        let missing_texture_gltf = fixture.test_data_path.join("missing_texture.gltf");
        let gltf_with_missing_texture = r#"{
            "scene": 0,
            "scenes": [{"nodes": [0]}],
            "nodes": [{"mesh": 0}],
            "meshes": [{
                "primitives": [{
                    "attributes": {"POSITION": 0},
                    "material": 0
                }]
            }],
            "materials": [{
                "pbrMetallicRoughness": {
                    "baseColorTexture": {
                        "index": 0,
                        "source": "missing_texture.png"
                    }
                }
            }],
            "asset": {"version": "2.0"}
        }"#;
        std::fs::write(&missing_texture_gltf, gltf_with_missing_texture).expect("Failed to write GLTF with missing texture");

        let result = fixture.asset_pipeline.import_asset(&missing_texture_gltf);

        // Should either fail or succeed with warnings about missing dependencies
        if let Ok(asset_id) = result {
            let asset = fixture.asset_database.get_asset(asset_id).expect("Asset should exist");
            assert!(asset.has_missing_dependencies(), "Asset should report missing dependencies");

            let missing_deps = asset.get_missing_dependencies();
            assert!(!missing_deps.is_empty(), "Should list missing dependencies");
            assert!(missing_deps.iter().any(|dep| dep.contains("missing_texture.png")));
        } else {
            assert!(matches!(result.unwrap_err(), AssetError::MissingDependency(_)));
        }

        // Test circular dependencies
        let (circular_a, circular_b) = fixture.create_circular_dependency_assets();

        let result_a = fixture.asset_pipeline.import_asset(&circular_a);
        let result_b = fixture.asset_pipeline.import_asset(&circular_b);

        // Should detect and handle circular dependencies
        if let (Ok(asset_a_id), Ok(asset_b_id)) = (result_a, result_b) {
            let dependency_graph = fixture.asset_database.get_dependency_graph();
            assert!(dependency_graph.has_circular_dependency(),
                "Should detect circular dependency");

            let cycles = dependency_graph.find_cycles();
            assert!(!cycles.is_empty(), "Should find dependency cycles");
            assert!(cycles.iter().any(|cycle| cycle.contains(&asset_a_id) && cycle.contains(&asset_b_id)));
        }
    }

    #[test]
    fn test_database_error_handling() {
        let mut fixture = ErrorTestFixture::new();

        // Create and import some assets first
        let test_asset_path = fixture.test_data_path.join("db_test.txt");
        std::fs::write(&test_asset_path, b"test content").expect("Failed to create test asset");
        let asset_id = fixture.asset_pipeline.import_asset(&test_asset_path).expect("Failed to import test asset");

        // Simulate database corruption
        fixture.simulate_database_corruption();

        // Test database operations after corruption
        let result = fixture.asset_database.get_asset(asset_id);
        assert!(result.is_err(), "Should fail to read from corrupted database");
        assert!(matches!(result.unwrap_err(), AssetError::DatabaseError(_)));

        // Test search operations
        let search_result = fixture.asset_database.search_assets("test");
        assert!(search_result.is_empty() || search_result.is_err(), "Search should fail or return empty results");

        // Test database recovery
        let recovery_result = fixture.asset_database.attempt_recovery();
        if recovery_result.is_ok() {
            // If recovery succeeds, operations should work again
            let test_result = fixture.asset_database.test_connection();
            assert!(test_result.is_ok(), "Database should work after recovery");
        } else {
            // If recovery fails, should be able to create new database
            let new_db_result = AssetDatabase::new("error_test_recovery.db");
            assert!(new_db_result.is_ok(), "Should be able to create new database");
        }

        // Test concurrent database access conflicts
        let handles: Vec<_> = (0..5).map(|i| {
            let database = fixture.asset_database.clone();
            let test_path = fixture.test_data_path.clone();
            std::thread::spawn(move || {
                let asset_path = test_path.join(format!("concurrent_{}.txt", i));
                std::fs::write(&asset_path, format!("content {}", i)).unwrap();
                database.import_asset(&asset_path)
            })
        }).collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // At least some operations should succeed despite conflicts
        let successful = results.iter().filter(|r| r.is_ok()).count();
        assert!(successful > 0, "At least some concurrent operations should succeed");

        // Failed operations should have appropriate error types
        for result in &results {
            if let Err(error) = result {
                assert!(matches!(error,
                    AssetError::DatabaseError(_) |
                    AssetError::ConcurrencyConflict(_) |
                    AssetError::LockTimeout(_)
                ));
            }
        }
    }

    #[test]
    fn test_ui_component_error_states() {
        let mut fixture = ErrorTestFixture::new();

        // Test button with missing icon asset
        let missing_icon_button = Button::new("Button with Missing Icon".to_string())
            .with_icon_asset("nonexistent_icon.png");

        let button_id = fixture.ui_manager.add_element("missing_icon_btn", Box::new(missing_icon_button));

        // Button should handle missing asset gracefully
        let button_element = fixture.ui_manager.get_element(button_id).expect("Button should exist");
        assert!(button_element.has_missing_assets(), "Button should detect missing icon");
        assert!(button_element.can_render_without_assets(), "Button should be able to render without icon");

        // Test rendering with missing assets
        let render_result = button_element.render();
        assert!(render_result.is_ok(), "Button should render despite missing icon");

        if let Ok(render_output) = render_result {
            assert!(render_output.has_fallback_content(), "Should use fallback rendering");
            assert!(!render_output.has_icon(), "Should not show icon");
        }

        // Test form with validation errors
        let mut form = Form::new("error_form".to_string());
        form.add_text_field("required_field", "Required Field", true, Some("This field is required".to_string()));
        form.add_email_field("email", "Email", true, Some("Please enter a valid email".to_string()));

        let form_id = fixture.ui_manager.add_element("error_form", Box::new(form));

        // Submit form with invalid data
        let mut form_data = HashMap::new();
        form_data.insert("required_field".to_string(), "".to_string()); // Empty required field
        form_data.insert("email".to_string(), "invalid-email".to_string()); // Invalid email

        let form_element = fixture.ui_manager.get_element_mut(form_id).expect("Form should exist");
        let validation_result = form_element.validate(&form_data);

        assert!(!validation_result.is_valid(), "Form validation should fail");
        assert!(!validation_result.errors.is_empty(), "Should have validation errors");

        // Form should display error states
        assert!(form_element.has_validation_errors(), "Form should show validation errors");
        assert!(form_element.is_in_error_state(), "Form should be in error state");

        // Test modal with broken content
        let mut modal = Modal::new("broken_modal".to_string(), "Broken Modal".to_string());
        modal.set_content_from_asset("nonexistent_content.html");

        let modal_id = fixture.ui_manager.add_element("broken_modal", Box::new(modal));

        let modal_element = fixture.ui_manager.get_element(modal_id).expect("Modal should exist");
        assert!(modal_element.has_missing_assets(), "Modal should detect missing content");

        // Modal should handle missing content gracefully
        let show_result = modal_element.show();
        assert!(show_result.is_ok(), "Modal should be able to show despite missing content");

        // Test navigation with broken links
        let mut nav = Navigation::new("broken_nav".to_string());
        nav.add_item("home", "Home", "/", None);
        nav.add_item("broken", "Broken Link", "/nonexistent", None);

        let nav_id = fixture.ui_manager.add_element("broken_nav", Box::new(nav));

        let nav_element = fixture.ui_manager.get_element_mut(nav_id).expect("Navigation should exist");

        // Test clicking broken link
        let click_event = UIEvent::Click {
            element_id: nav_id,
            position: (100.0, 50.0),
        };

        let click_result = nav_element.handle_event(&click_event);

        // Should handle broken navigation gracefully
        if let Err(error) = click_result {
            assert!(matches!(error, UIError::NavigationError(_)));
            assert!(nav_element.is_in_error_state(), "Navigation should be in error state");
            assert!(nav_element.has_error_message(), "Should display error message to user");
        }
    }

    #[test]
    fn test_memory_pressure_handling() {
        let mut fixture = ErrorTestFixture::new();

        // Simulate low memory conditions
        fixture.simulate_memory_pressure();

        // Try to import assets under memory pressure
        let asset_paths: Vec<_> = (0..50).map(|i| {
            let path = fixture.test_data_path.join(format!("memory_test_{}.txt", i));
            std::fs::write(&path, vec![b'A'; 1024 * 50]).expect("Failed to create large test file"); // 50KB each
            path
        }).collect();

        let mut successful_imports = 0;
        let mut memory_errors = 0;

        for path in &asset_paths {
            match fixture.asset_pipeline.import_asset(path) {
                Ok(_) => successful_imports += 1,
                Err(AssetError::OutOfMemory(_)) => memory_errors += 1,
                Err(_) => {} // Other errors
            }
        }

        // Should handle memory pressure gracefully
        assert!(memory_errors > 0, "Should encounter memory pressure errors");
        assert!(successful_imports > 0, "Some imports should still succeed");

        // Test memory cleanup under pressure
        let initial_memory = fixture.engine.get_memory_usage();
        fixture.asset_pipeline.cleanup_unused_assets();
        let after_cleanup_memory = fixture.engine.get_memory_usage();

        assert!(after_cleanup_memory <= initial_memory,
            "Memory usage should not increase after cleanup");

        // Test UI behavior under memory pressure
        let mut ui_creation_errors = 0;
        for i in 0..100 {
            let button = Button::new(format!("Memory Test Button {}", i));
            match fixture.ui_manager.add_element(&format!("memory_btn_{}", i), Box::new(button)) {
                Ok(_) => {}
                Err(UIError::OutOfMemory(_)) => ui_creation_errors += 1,
                Err(_) => {} // Other errors
            }
        }

        // UI should handle memory pressure
        if ui_creation_errors > 0 {
            assert!(ui_creation_errors < 100, "Not all UI creation should fail");

            // Test memory recovery for UI
            fixture.ui_manager.cleanup_unused_elements();
            let button_after_cleanup = Button::new("Recovery Test Button".to_string());
            let recovery_result = fixture.ui_manager.add_element("recovery_btn", Box::new(button_after_cleanup));

            assert!(recovery_result.is_ok(), "Should be able to create UI elements after cleanup");
        }
    }

    #[test]
    fn test_concurrent_access_conflicts() {
        let mut fixture = ErrorTestFixture::new();

        // Create shared asset for concurrent access
        let shared_asset_path = fixture.test_data_path.join("shared_asset.txt");
        std::fs::write(&shared_asset_path, b"shared content").expect("Failed to create shared asset");
        let shared_asset_id = fixture.asset_pipeline.import_asset(&shared_asset_path).expect("Failed to import shared asset");

        // Test concurrent read/write operations
        let handles: Vec<_> = (0..10).map(|i| {
            let asset_database = fixture.asset_database.clone();
            let asset_pipeline = fixture.asset_pipeline.clone();
            let test_path = fixture.test_data_path.clone();

            std::thread::spawn(move || {
                let operation_type = i % 3;
                match operation_type {
                    0 => {
                        // Read operations
                        asset_database.get_asset(shared_asset_id)
                    }
                    1 => {
                        // Write operations (metadata update)
                        asset_database.set_asset_metadata(shared_asset_id, &format!("key_{}", i), &format!("value_{}", i))
                            .map(|_| ()) // Convert to compatible type
                            .map_err(|e| AssetError::DatabaseError(format!("Metadata update failed: {:?}", e)))
                    }
                    2 => {
                        // Import new assets
                        let new_asset_path = test_path.join(format!("concurrent_asset_{}.txt", i));
                        std::fs::write(&new_asset_path, format!("content {}", i)).unwrap();
                        asset_pipeline.import_asset(&new_asset_path)
                            .map(|_| ()) // Convert to compatible type
                    }
                    _ => unreachable!()
                }
            })
        }).collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // Analyze results
        let successful_operations = results.iter().filter(|r| r.is_ok()).count();
        let conflicts = results.iter().filter(|r| {
            if let Err(error) = r {
                matches!(error, AssetError::ConcurrencyConflict(_) | AssetError::LockTimeout(_))
            } else {
                false
            }
        }).count();

        assert!(successful_operations > 0, "Some concurrent operations should succeed");

        if conflicts > 0 {
            println!("Detected {} concurrency conflicts out of {} operations", conflicts, results.len());
            // Conflicts are acceptable, but system should handle them gracefully
        }

        // Test UI concurrent access
        let ui_handles: Vec<_> = (0..5).map(|i| {
            let ui_manager = fixture.ui_manager.clone();
            std::thread::spawn(move || {
                for j in 0..10 {
                    let button = Button::new(format!("Concurrent Button {}-{}", i, j));
                    ui_manager.add_element(&format!("concurrent_{}_{}", i, j), Box::new(button))
                }
            })
        }).collect();

        let ui_results: Vec<_> = ui_handles.into_iter().map(|h| h.join().unwrap()).collect();

        // UI operations should generally succeed
        let ui_errors = ui_results.iter().filter(|r| r.is_err()).count();
        assert!(ui_errors < ui_results.len() / 2, "Most UI operations should succeed");
    }

    #[test]
    fn test_malformed_configuration_handling() {
        let mut fixture = ErrorTestFixture::new();

        // Test malformed theme configuration
        let malformed_theme_config = r#"{
            "name": "BrokenTheme",
            "colors": {
                "primary": "not_a_color",
                "secondary": "#invalid_hex",
                "background": 12345
            },
            "fonts": {
                "size": -10,
                "family": null
            }
        }"#;

        let theme_config_path = fixture.test_data_path.join("broken_theme.json");
        std::fs::write(&theme_config_path, malformed_theme_config).expect("Failed to write broken theme config");

        let theme_load_result = fixture.theme_engine.load_theme_from_file(&theme_config_path);
        assert!(theme_load_result.is_err(), "Should reject malformed theme configuration");

        // Theme engine should still be functional after error
        let default_theme_result = fixture.theme_engine.get_default_theme();
        assert!(default_theme_result.is_ok(), "Should be able to get default theme after error");

        // Test malformed asset pipeline configuration
        let malformed_pipeline_config = r#"{
            "importers": {
                "png": {
                    "max_size": "not_a_number",
                    "quality": 150
                }
            },
            "cache": {
                "size": -1,
                "location": "/invalid/path/that/does/not/exist"
            }
        }"#;

        let pipeline_config_path = fixture.test_data_path.join("broken_pipeline.json");
        std::fs::write(&pipeline_config_path, malformed_pipeline_config).expect("Failed to write broken pipeline config");

        let pipeline_config_result = fixture.asset_pipeline.load_configuration(&pipeline_config_path);
        assert!(pipeline_config_result.is_err(), "Should reject malformed pipeline configuration");

        // Pipeline should use default configuration after error
        let pipeline_status = fixture.asset_pipeline.get_status();
        assert!(pipeline_status.is_configured(), "Pipeline should fall back to default configuration");

        // Test UI configuration errors
        let malformed_ui_config = r#"{
            "window": {
                "width": -800,
                "height": "not_a_number"
            },
            "components": {
                "button": {
                    "default_size": "invalid_size",
                    "animations": true
                }
            }
        }"#;

        let ui_config_path = fixture.test_data_path.join("broken_ui.json");
        std::fs::write(&ui_config_path, malformed_ui_config).expect("Failed to write broken UI config");

        let ui_config_result = fixture.ui_manager.load_configuration(&ui_config_path);
        assert!(ui_config_result.is_err(), "Should reject malformed UI configuration");

        // UI should still be functional with default configuration
        let test_button = Button::new("Test After Config Error".to_string());
        let button_result = fixture.ui_manager.add_element("test_after_error", Box::new(test_button));
        assert!(button_result.is_ok(), "UI should work with default configuration after error");
    }

    #[test]
    fn test_security_and_input_validation() {
        let mut fixture = ErrorTestFixture::new();

        // Test path traversal attempts
        let malicious_paths = vec![
            "../../../etc/passwd",
            "..\\..\\windows\\system32\\config\\sam",
            "/etc/passwd",
            "C:\\Windows\\System32\\config\\SAM",
            "assets/../../../secret.txt",
        ];

        for malicious_path in malicious_paths {
            let result = fixture.asset_pipeline.import_asset(&PathBuf::from(malicious_path));
            assert!(result.is_err(), "Should reject path traversal attempt: {}", malicious_path);

            if let Err(error) = result {
                assert!(matches!(error,
                    AssetError::SecurityViolation(_) |
                    AssetError::InvalidPath(_) |
                    AssetError::FileNotFound(_)
                ));
            }
        }

        // Test oversized inputs
        let extremely_long_name = "a".repeat(10000);
        let button_with_long_name = Button::new(extremely_long_name.clone());

        let result = fixture.ui_manager.add_element(&extremely_long_name, Box::new(button_with_long_name));

        // Should either truncate the name or reject it
        if let Err(error) = result {
            assert!(matches!(error, UIError::InvalidInput(_) | UIError::InputTooLong(_)));
        } else {
            let element = fixture.ui_manager.get_element(result.unwrap()).expect("Element should exist");
            let display_name = element.get_display_name();
            assert!(display_name.len() <= 1000, "Display name should be truncated");
        }

        // Test script injection attempts in UI text
        let malicious_texts = vec![
            "<script>alert('xss')</script>",
            "javascript:alert('xss')",
            "${this.evil_function()}",
            "../../assets/../../etc/passwd",
        ];

        for malicious_text in malicious_texts {
            let button = Button::new(malicious_text.to_string());
            let button_id = fixture.ui_manager.add_element("malicious_btn", Box::new(button)).expect("Button should be created");

            let element = fixture.ui_manager.get_element(button_id).expect("Element should exist");
            let sanitized_text = element.get_sanitized_text();

            // Text should be sanitized
            assert!(!sanitized_text.contains("<script>"), "Script tags should be removed");
            assert!(!sanitized_text.contains("javascript:"), "JavaScript URLs should be removed");
            assert!(!sanitized_text.contains("${"), "Template expressions should be escaped");

            fixture.ui_manager.remove_element(button_id);
        }

        // Test SQL injection attempts in asset search
        let sql_injection_attempts = vec![
            "'; DROP TABLE assets; --",
            "1' OR '1'='1",
            "UNION SELECT * FROM users",
            "<script>alert('xss')</script>",
        ];

        for injection_attempt in sql_injection_attempts {
            let search_result = fixture.asset_database.search_assets(injection_attempt);

            // Should return empty results or safe results, not error
            match search_result {
                Ok(results) => {
                    // Results should be empty or safe
                    for asset_id in results {
                        let asset = fixture.asset_database.get_asset(asset_id).expect("Asset should exist");
                        assert!(!asset.name.contains(injection_attempt),
                            "Search should not return malicious content");
                    }
                }
                Err(error) => {
                    // Should be a safe error, not a database error that leaks information
                    assert!(matches!(error, AssetError::InvalidQuery(_)));
                }
            }
        }

        // Test file upload size limits
        if let Ok(()) = fixture.create_extremely_large_file(&fixture.test_data_path.join("huge_file.txt")) {
            let result = fixture.asset_pipeline.import_asset(&fixture.test_data_path.join("huge_file.txt"));
            assert!(result.is_err(), "Should reject extremely large files");

            if let Err(error) = result {
                assert!(matches!(error, AssetError::FileTooLarge(_) | AssetError::OutOfMemory(_)));
            }
        }
    }

    #[test]
    fn test_error_recovery_mechanisms() {
        let mut fixture = ErrorTestFixture::new();

        // Test automatic retry mechanism
        let flaky_asset_path = fixture.test_data_path.join("flaky_asset.txt");
        std::fs::write(&flaky_asset_path, b"flaky content").expect("Failed to create flaky asset");

        // Simulate flaky import that fails first time but succeeds on retry
        fixture.asset_pipeline.simulate_flaky_operations(true);

        let result = fixture.asset_pipeline.import_asset_with_retry(&flaky_asset_path, 3);
        assert!(result.is_ok(), "Should succeed with retry mechanism");

        // Test graceful degradation
        fixture.simulate_database_corruption();

        // UI should continue working even with database issues
        let button = Button::new("Degraded Mode Button".to_string());
        let button_result = fixture.ui_manager.add_element("degraded_btn", Box::new(button));
        assert!(button_result.is_ok(), "UI should work in degraded mode");

        // Asset search should degrade gracefully
        let search_result = fixture.asset_database.search_assets("test");
        // Should not crash, either return empty results or cached results
        match search_result {
            Ok(results) => assert!(results.is_empty(), "Should return empty results in degraded mode"),
            Err(_) => {} // Error is also acceptable in degraded mode
        }

        // Test error reporting and logging
        let error_reports = fixture.engine.get_error_reports();
        assert!(!error_reports.is_empty(), "Should collect error reports");

        for report in &error_reports {
            assert!(!report.message.is_empty(), "Error reports should have messages");
            assert!(report.timestamp > 0, "Error reports should have timestamps");
            assert!(!report.context.is_empty(), "Error reports should have context");
        }

        // Test user notification of errors
        let ui_notifications = fixture.ui_manager.get_error_notifications();
        assert!(!ui_notifications.is_empty(), "Should show error notifications to user");

        for notification in &ui_notifications {
            assert!(notification.is_user_friendly(), "Notifications should be user-friendly");
            assert!(!notification.contains_technical_details(), "Should not expose technical details to user");
        }
    }
}

/// Mock error types and implementations for testing
#[cfg(test)]
mod error_mocks {
    use super::*;

    #[derive(Debug, Clone)]
    pub enum UIError {
        OutOfMemory(String),
        NavigationError(String),
        InvalidInput(String),
        InputTooLong(String),
    }

    #[derive(Debug, Clone)]
    pub struct ErrorReport {
        pub message: String,
        pub timestamp: u64,
        pub context: String,
        pub severity: ErrorSeverity,
    }

    #[derive(Debug, Clone)]
    pub enum ErrorSeverity {
        Low,
        Medium,
        High,
        Critical,
    }

    #[derive(Debug, Clone)]
    pub struct UserNotification {
        message: String,
        notification_type: NotificationType,
        auto_dismiss: bool,
    }

    impl UserNotification {
        pub fn is_user_friendly(&self) -> bool {
            // Check if message is user-friendly (no technical jargon)
            !self.message.contains("SQL") &&
            !self.message.contains("database") &&
            !self.message.contains("exception") &&
            !self.message.contains("stack trace")
        }

        pub fn contains_technical_details(&self) -> bool {
            self.message.contains("::") || // Rust paths
            self.message.contains("0x") || // Memory addresses
            self.message.contains("error code") ||
            self.message.contains("errno")
        }
    }

    #[derive(Debug, Clone)]
    pub enum NotificationType {
        Info,
        Warning,
        Error,
    }

    pub trait MockAssetDatabaseExtensions {
        fn get_database_path(&self) -> Option<PathBuf>;
        fn attempt_recovery(&self) -> Result<(), AssetError>;
        fn test_connection(&self) -> Result<(), AssetError>;
        fn get_dependency_graph(&self) -> DependencyGraph;
        fn import_asset(&self, path: &PathBuf) -> Result<AssetId, AssetError>;
        fn set_asset_metadata(&self, asset_id: AssetId, key: &str, value: &str) -> Result<(), AssetError>;
    }

    pub struct DependencyGraph {
        dependencies: HashMap<AssetId, Vec<AssetId>>,
    }

    impl DependencyGraph {
        pub fn has_circular_dependency(&self) -> bool {
            // Mock implementation - in real code this would do cycle detection
            true
        }

        pub fn find_cycles(&self) -> Vec<Vec<AssetId>> {
            // Mock implementation
            vec![vec![AssetId::new(1), AssetId::new(2)]]
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AssetId(u32);

    impl AssetId {
        pub fn new(id: u32) -> Self {
            Self(id)
        }
    }
}