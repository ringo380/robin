use robin::engine::assets::{
    AssetDatabase, AssetImporter, AssetPipeline, HotReloadSystem,
    importers::{GLTFImporter, FBXImporter, OBJImporter, TextureImporter, AudioImporter},
    advanced_pipeline::{TextureCompression, QualityMetrics, PlatformOptimization},
};
use std::path::PathBuf;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Comprehensive Asset Pipeline Enhancement Test Suite
///
/// This test suite validates the asset pipeline for:
/// - Multi-format importer functionality (GLTF, FBX, OBJ, textures, audio)
/// - Asset database operations (search, dependencies, collections)
/// - Hot reload system reliability
/// - Platform optimization workflows
/// - Texture compression and quality metrics
/// - Performance characteristics
/// - Error handling for edge cases

#[cfg(test)]
mod asset_pipeline_tests {
    use super::*;

    /// Test fixture for asset pipeline testing
    struct AssetTestFixture {
        pipeline: AssetPipeline,
        database: AssetDatabase,
        hot_reload: HotReloadSystem,
        test_assets_path: PathBuf,
    }

    impl AssetTestFixture {
        fn new() -> Self {
            let test_assets_path = PathBuf::from("tests/assets");
            std::fs::create_dir_all(&test_assets_path).expect("Failed to create test assets directory");

            let database = AssetDatabase::new("test_assets.db").expect("Failed to create test database");
            let pipeline = AssetPipeline::new(database.clone());
            let hot_reload = HotReloadSystem::new(&test_assets_path);

            Self {
                pipeline,
                database,
                hot_reload,
                test_assets_path,
            }
        }

        fn create_test_texture(&self, name: &str, format: &str) -> PathBuf {
            let file_path = self.test_assets_path.join(format!("{}.{}", name, format));
            // Create minimal test texture file
            match format {
                "png" => self.create_test_png(&file_path),
                "jpg" => self.create_test_jpg(&file_path),
                "tga" => self.create_test_tga(&file_path),
                _ => panic!("Unsupported test texture format: {}", format),
            }
            file_path
        }

        fn create_test_model(&self, name: &str, format: &str) -> PathBuf {
            let file_path = self.test_assets_path.join(format!("{}.{}", name, format));
            // Create minimal test model file
            match format {
                "gltf" => self.create_test_gltf(&file_path),
                "fbx" => self.create_test_fbx(&file_path),
                "obj" => self.create_test_obj(&file_path),
                _ => panic!("Unsupported test model format: {}", format),
            }
            file_path
        }

        fn create_test_audio(&self, name: &str, format: &str) -> PathBuf {
            let file_path = self.test_assets_path.join(format!("{}.{}", name, format));
            // Create minimal test audio file
            match format {
                "wav" => self.create_test_wav(&file_path),
                "ogg" => self.create_test_ogg(&file_path),
                "mp3" => self.create_test_mp3(&file_path),
                _ => panic!("Unsupported test audio format: {}", format),
            }
            file_path
        }

        // Helper methods for creating test files
        fn create_test_png(&self, path: &PathBuf) {
            // Create a minimal 2x2 PNG for testing
            let png_data = vec![
                0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
                // IHDR chunk
                0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
                0x00, 0x00, 0x00, 0x02, 0x00, 0x00, 0x00, 0x02,
                0x08, 0x06, 0x00, 0x00, 0x00, 0x72, 0xB6, 0x0D,
                0x24,
                // IDAT chunk (minimal)
                0x00, 0x00, 0x00, 0x16, 0x49, 0x44, 0x41, 0x54,
                0x78, 0x9C, 0x63, 0x60, 0x60, 0x60, 0xF8, 0x0F,
                0x00, 0x01, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
                0xB4, 0x1C, 0x68, 0x3E, 0xD8,
                // IEND chunk
                0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
                0xAE, 0x42, 0x60, 0x82,
            ];
            std::fs::write(path, png_data).expect("Failed to write test PNG");
        }

        fn create_test_gltf(&self, path: &PathBuf) {
            let gltf_json = r#"{
                "scene": 0,
                "scenes": [{"nodes": [0]}],
                "nodes": [{"mesh": 0}],
                "meshes": [{
                    "primitives": [{
                        "attributes": {"POSITION": 0},
                        "indices": 1
                    }]
                }],
                "buffers": [{"byteLength": 144}],
                "bufferViews": [
                    {"buffer": 0, "byteOffset": 0, "byteLength": 96, "target": 34962},
                    {"buffer": 0, "byteOffset": 96, "byteLength": 48, "target": 34963}
                ],
                "accessors": [
                    {"bufferView": 0, "componentType": 5126, "count": 8, "type": "VEC3"},
                    {"bufferView": 1, "componentType": 5123, "count": 12, "type": "SCALAR"}
                ],
                "asset": {"version": "2.0"}
            }"#;
            std::fs::write(path, gltf_json).expect("Failed to write test GLTF");
        }

        fn create_test_obj(&self, path: &PathBuf) {
            let obj_content = "# Simple test cube\nv 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 1.0 1.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3 4\n";
            std::fs::write(path, obj_content).expect("Failed to write test OBJ");
        }

        fn create_test_wav(&self, path: &PathBuf) {
            // Minimal WAV header for 1 second of silence at 44.1kHz mono
            let wav_data = vec![
                // RIFF header
                0x52, 0x49, 0x46, 0x46, 0x24, 0x08, 0x00, 0x00,
                0x57, 0x41, 0x56, 0x45, 0x66, 0x6D, 0x74, 0x20,
                0x10, 0x00, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
                0x44, 0xAC, 0x00, 0x00, 0x88, 0x58, 0x01, 0x00,
                0x02, 0x00, 0x10, 0x00, 0x64, 0x61, 0x74, 0x61,
                0x00, 0x08, 0x00, 0x00,
                // Silence data (2048 bytes of zeros for simplicity)
            ];
            let mut full_wav = wav_data;
            full_wav.extend(vec![0u8; 2048]); // Add silence data
            std::fs::write(path, full_wav).expect("Failed to write test WAV");
        }

        // Placeholder implementations for other formats
        fn create_test_jpg(&self, _path: &PathBuf) { /* Implement minimal JPEG */ }
        fn create_test_tga(&self, _path: &PathBuf) { /* Implement minimal TGA */ }
        fn create_test_fbx(&self, _path: &PathBuf) { /* Implement minimal FBX */ }
        fn create_test_ogg(&self, _path: &PathBuf) { /* Implement minimal OGG */ }
        fn create_test_mp3(&self, _path: &PathBuf) { /* Implement minimal MP3 */ }
    }

    impl Drop for AssetTestFixture {
        fn drop(&mut self) {
            // Clean up test files
            let _ = std::fs::remove_dir_all(&self.test_assets_path);
        }
    }

    #[test]
    fn test_gltf_importer_functionality() {
        let mut fixture = AssetTestFixture::new();
        let gltf_path = fixture.create_test_model("test_cube", "gltf");

        let importer = GLTFImporter::new();
        let import_result = importer.import(&gltf_path);

        assert!(import_result.is_ok(), "GLTF import failed: {:?}", import_result.err());

        let asset = import_result.unwrap();
        assert_eq!(asset.asset_type, AssetType::Model);
        assert!(!asset.meshes.is_empty(), "GLTF should contain at least one mesh");
        assert!(!asset.materials.is_empty(), "GLTF should contain at least one material");

        // Test metadata extraction
        assert!(asset.metadata.contains_key("vertex_count"));
        assert!(asset.metadata.contains_key("triangle_count"));
        assert!(asset.metadata.contains_key("material_count"));

        // Test dependency tracking
        if !asset.dependencies.is_empty() {
            for dependency in &asset.dependencies {
                assert!(dependency.exists(), "Dependency file should exist: {:?}", dependency);
            }
        }
    }

    #[test]
    fn test_fbx_importer_functionality() {
        let mut fixture = AssetTestFixture::new();
        let fbx_path = fixture.create_test_model("test_character", "fbx");

        let importer = FBXImporter::new();
        let import_result = importer.import(&fbx_path);

        assert!(import_result.is_ok(), "FBX import failed: {:?}", import_result.err());

        let asset = import_result.unwrap();
        assert_eq!(asset.asset_type, AssetType::Model);

        // Test animation support
        if asset.has_animations() {
            assert!(!asset.animations.is_empty());
            for animation in &asset.animations {
                assert!(animation.duration > 0.0);
                assert!(!animation.channels.is_empty());
            }
        }

        // Test skeleton support
        if asset.has_skeleton() {
            assert!(asset.skeleton.is_some());
            let skeleton = asset.skeleton.as_ref().unwrap();
            assert!(!skeleton.bones.is_empty());
        }
    }

    #[test]
    fn test_obj_importer_functionality() {
        let mut fixture = AssetTestFixture::new();
        let obj_path = fixture.create_test_model("test_object", "obj");

        let importer = OBJImporter::new();
        let import_result = importer.import(&obj_path);

        assert!(import_result.is_ok(), "OBJ import failed: {:?}", import_result.err());

        let asset = import_result.unwrap();
        assert_eq!(asset.asset_type, AssetType::Model);
        assert!(!asset.meshes.is_empty(), "OBJ should contain at least one mesh");

        // Test mesh properties
        let mesh = &asset.meshes[0];
        assert!(!mesh.vertices.is_empty(), "Mesh should have vertices");
        assert!(!mesh.indices.is_empty(), "Mesh should have indices");

        // Test material loading (.mtl file support)
        if obj_path.with_extension("mtl").exists() {
            assert!(!asset.materials.is_empty(), "Should load materials from .mtl file");
        }
    }

    #[test]
    fn test_texture_importer_functionality() {
        let mut fixture = AssetTestFixture::new();

        // Test multiple texture formats
        let formats = vec!["png", "jpg", "tga"];
        for format in formats {
            let texture_path = fixture.create_test_texture(&format!("test_{}", format), format);
            let importer = TextureImporter::new();
            let import_result = importer.import(&texture_path);

            assert!(import_result.is_ok(), "Texture import failed for {}: {:?}", format, import_result.err());

            let asset = import_result.unwrap();
            assert_eq!(asset.asset_type, AssetType::Texture);
            assert!(asset.width > 0, "Texture should have valid width");
            assert!(asset.height > 0, "Texture should have valid height");
            assert!(!asset.pixel_data.is_empty(), "Texture should have pixel data");

            // Test format-specific properties
            match format {
                "png" => assert!(asset.has_alpha_channel()),
                "jpg" => assert!(!asset.has_alpha_channel()),
                "tga" => {
                    // TGA can have alpha depending on file
                    assert!(asset.bits_per_pixel == 24 || asset.bits_per_pixel == 32);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_audio_importer_functionality() {
        let mut fixture = AssetTestFixture::new();

        // Test multiple audio formats
        let formats = vec!["wav", "ogg", "mp3"];
        for format in formats {
            let audio_path = fixture.create_test_audio(&format!("test_{}", format), format);
            let importer = AudioImporter::new();
            let import_result = importer.import(&audio_path);

            assert!(import_result.is_ok(), "Audio import failed for {}: {:?}", format, import_result.err());

            let asset = import_result.unwrap();
            assert_eq!(asset.asset_type, AssetType::Audio);
            assert!(asset.sample_rate > 0, "Audio should have valid sample rate");
            assert!(asset.duration > 0.0, "Audio should have valid duration");
            assert!(!asset.audio_data.is_empty(), "Audio should have sample data");

            // Test format-specific properties
            match format {
                "wav" => {
                    assert_eq!(asset.format, AudioFormat::WAV);
                    assert!(asset.is_uncompressed());
                }
                "ogg" => {
                    assert_eq!(asset.format, AudioFormat::OGG);
                    assert!(asset.is_compressed());
                }
                "mp3" => {
                    assert_eq!(asset.format, AudioFormat::MP3);
                    assert!(asset.is_compressed());
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_asset_database_operations() {
        let mut fixture = AssetTestFixture::new();

        // Create test assets
        let texture_path = fixture.create_test_texture("db_test", "png");
        let model_path = fixture.create_test_model("db_test", "gltf");

        // Import assets into database
        let texture_id = fixture.pipeline.import_asset(&texture_path).expect("Failed to import texture");
        let model_id = fixture.pipeline.import_asset(&model_path).expect("Failed to import model");

        // Test asset retrieval
        let texture_asset = fixture.database.get_asset(texture_id).expect("Failed to get texture asset");
        let model_asset = fixture.database.get_asset(model_id).expect("Failed to get model asset");

        assert_eq!(texture_asset.asset_type, AssetType::Texture);
        assert_eq!(model_asset.asset_type, AssetType::Model);

        // Test search functionality
        let search_results = fixture.database.search_assets("db_test");
        assert_eq!(search_results.len(), 2, "Should find both test assets");

        // Test filtering by type
        let texture_results = fixture.database.search_assets_by_type(AssetType::Texture);
        assert!(texture_results.contains(&texture_id), "Should find texture in type filter");

        let model_results = fixture.database.search_assets_by_type(AssetType::Model);
        assert!(model_results.contains(&model_id), "Should find model in type filter");

        // Test metadata queries
        let large_textures = fixture.database.query_assets_with_metadata("width", ">1024");
        let small_textures = fixture.database.query_assets_with_metadata("width", "<=1024");

        // Test dependency tracking
        if !model_asset.dependencies.is_empty() {
            let dependents = fixture.database.get_asset_dependents(texture_id);
            if model_asset.dependencies.contains(&texture_path) {
                assert!(dependents.contains(&model_id), "Model should depend on texture");
            }
        }
    }

    #[test]
    fn test_asset_collections_management() {
        let mut fixture = AssetTestFixture::new();

        // Create test assets
        let texture1 = fixture.create_test_texture("collection_test_1", "png");
        let texture2 = fixture.create_test_texture("collection_test_2", "png");
        let model1 = fixture.create_test_model("collection_test_1", "gltf");

        let texture1_id = fixture.pipeline.import_asset(&texture1).expect("Failed to import texture1");
        let texture2_id = fixture.pipeline.import_asset(&texture2).expect("Failed to import texture2");
        let model1_id = fixture.pipeline.import_asset(&model1).expect("Failed to import model1");

        // Create collections
        let character_collection = fixture.database.create_collection("character_assets", "Assets for character system");
        let environment_collection = fixture.database.create_collection("environment_assets", "Assets for environment");

        // Add assets to collections
        fixture.database.add_asset_to_collection(character_collection, texture1_id);
        fixture.database.add_asset_to_collection(character_collection, model1_id);
        fixture.database.add_asset_to_collection(environment_collection, texture2_id);

        // Test collection queries
        let character_assets = fixture.database.get_collection_assets(character_collection);
        assert_eq!(character_assets.len(), 2, "Character collection should have 2 assets");
        assert!(character_assets.contains(&texture1_id));
        assert!(character_assets.contains(&model1_id));

        let environment_assets = fixture.database.get_collection_assets(environment_collection);
        assert_eq!(environment_assets.len(), 1, "Environment collection should have 1 asset");
        assert!(environment_assets.contains(&texture2_id));

        // Test collection metadata
        let collections = fixture.database.get_asset_collections(texture1_id);
        assert!(collections.contains(&character_collection), "Texture1 should be in character collection");
    }

    #[test]
    fn test_hot_reload_system() {
        let mut fixture = AssetTestFixture::new();

        // Create and import initial asset
        let texture_path = fixture.create_test_texture("hot_reload_test", "png");
        let asset_id = fixture.pipeline.import_asset(&texture_path).expect("Failed to import initial asset");

        // Start hot reload monitoring
        fixture.hot_reload.start_monitoring();

        // Set up change detection
        let mut changes_detected = 0;
        fixture.hot_reload.set_change_callback(Box::new(|changed_files| {
            changes_detected += changed_files.len();
        }));

        // Simulate file modification
        std::thread::sleep(Duration::from_millis(100)); // Ensure different timestamp
        fixture.create_test_texture("hot_reload_test", "png"); // Recreate with new timestamp

        // Wait for file system notification
        std::thread::sleep(Duration::from_millis(500));

        assert!(changes_detected > 0, "Hot reload should detect file changes");

        // Test automatic reimport
        let reimported_asset = fixture.database.get_asset(asset_id).expect("Failed to get reimported asset");
        assert!(reimported_asset.last_modified > fixture.database.get_asset_import_time(asset_id));

        // Test selective monitoring
        fixture.hot_reload.add_file_filter("*.png");
        fixture.hot_reload.ignore_file_pattern("*_temp*");

        // Create files that should and shouldn't trigger reload
        let monitored_file = fixture.create_test_texture("monitored", "png");
        let ignored_file = fixture.test_assets_path.join("ignored_temp.png");
        std::fs::write(&ignored_file, b"ignored").expect("Failed to create ignored file");

        std::thread::sleep(Duration::from_millis(500));

        // Only the monitored file should trigger reload
        assert!(fixture.hot_reload.is_file_monitored(&monitored_file));
        assert!(!fixture.hot_reload.is_file_monitored(&ignored_file));
    }

    #[test]
    fn test_texture_compression_system() {
        let mut fixture = AssetTestFixture::new();
        let texture_path = fixture.create_test_texture("compression_test", "png");

        let importer = TextureImporter::new();
        let mut asset = importer.import(&texture_path).expect("Failed to import texture");

        // Test different compression formats
        let compression_formats = vec![
            TextureCompression::DXT1,
            TextureCompression::DXT5,
            TextureCompression::BC7,
            TextureCompression::ASTC,
        ];

        for format in compression_formats {
            let compressed = asset.compress_with_format(format);
            assert!(compressed.is_ok(), "Compression with {:?} failed: {:?}", format, compressed.err());

            let compressed_asset = compressed.unwrap();
            assert!(compressed_asset.is_compressed());
            assert_eq!(compressed_asset.compression_format, Some(format));

            // Verify compression ratio
            let original_size = asset.pixel_data.len();
            let compressed_size = compressed_asset.pixel_data.len();
            let compression_ratio = compressed_size as f32 / original_size as f32;

            match format {
                TextureCompression::DXT1 => assert!(compression_ratio < 0.5, "DXT1 should achieve good compression"),
                TextureCompression::DXT5 => assert!(compression_ratio < 0.7, "DXT5 should achieve moderate compression"),
                TextureCompression::BC7 => assert!(compression_ratio < 0.8, "BC7 should balance quality and size"),
                TextureCompression::ASTC => assert!(compression_ratio < 0.6, "ASTC should achieve good compression"),
            }
        }
    }

    #[test]
    fn test_quality_metrics_system() {
        let mut fixture = AssetTestFixture::new();

        // Test texture quality metrics
        let texture_path = fixture.create_test_texture("quality_test", "png");
        let texture_asset = TextureImporter::new().import(&texture_path).expect("Failed to import texture");

        let quality_metrics = QualityMetrics::analyze_texture(&texture_asset);
        assert!(quality_metrics.resolution_score >= 0.0 && quality_metrics.resolution_score <= 1.0);
        assert!(quality_metrics.compression_efficiency >= 0.0 && quality_metrics.compression_efficiency <= 1.0);
        assert!(quality_metrics.visual_quality >= 0.0 && quality_metrics.visual_quality <= 1.0);

        // Test model quality metrics
        let model_path = fixture.create_test_model("quality_test", "gltf");
        let model_asset = GLTFImporter::new().import(&model_path).expect("Failed to import model");

        let quality_metrics = QualityMetrics::analyze_model(&model_asset);
        assert!(quality_metrics.polygon_efficiency >= 0.0);
        assert!(quality_metrics.texture_usage >= 0.0 && quality_metrics.texture_usage <= 1.0);
        assert!(quality_metrics.material_complexity >= 0.0);

        // Test quality recommendations
        let recommendations = quality_metrics.get_optimization_recommendations();
        assert!(!recommendations.is_empty(), "Should provide optimization recommendations");

        for recommendation in recommendations {
            assert!(!recommendation.description.is_empty());
            assert!(recommendation.impact_score >= 0.0 && recommendation.impact_score <= 1.0);
            assert!(recommendation.difficulty >= 1 && recommendation.difficulty <= 5);
        }
    }

    #[test]
    fn test_platform_optimization_workflows() {
        let mut fixture = AssetTestFixture::new();

        // Create test assets
        let texture_path = fixture.create_test_texture("platform_test", "png");
        let model_path = fixture.create_test_model("platform_test", "gltf");

        let texture_id = fixture.pipeline.import_asset(&texture_path).expect("Failed to import texture");
        let model_id = fixture.pipeline.import_asset(&model_path).expect("Failed to import model");

        // Test platform-specific optimizations
        let platforms = vec![
            PlatformTarget::Desktop,
            PlatformTarget::Mobile,
            PlatformTarget::Web,
            PlatformTarget::Console,
        ];

        for platform in platforms {
            let optimization_profile = PlatformOptimization::get_profile(platform);

            // Test texture optimization for platform
            let optimized_texture = fixture.pipeline.optimize_for_platform(texture_id, platform);
            assert!(optimized_texture.is_ok(), "Texture optimization failed for {:?}", platform);

            let texture_variant = optimized_texture.unwrap();
            match platform {
                PlatformTarget::Mobile => {
                    assert!(texture_variant.max_resolution <= 1024, "Mobile textures should be limited in resolution");
                    assert!(texture_variant.is_compressed(), "Mobile textures should be compressed");
                }
                PlatformTarget::Web => {
                    assert!(texture_variant.format_preferences.contains(&TextureFormat::WebP), "Web should prefer WebP");
                }
                PlatformTarget::Console => {
                    assert!(texture_variant.quality_level >= QualityLevel::High, "Console should use high quality");
                }
                PlatformTarget::Desktop => {
                    assert!(texture_variant.quality_level >= QualityLevel::Medium, "Desktop should use medium+ quality");
                }
            }

            // Test model optimization for platform
            let optimized_model = fixture.pipeline.optimize_for_platform(model_id, platform);
            assert!(optimized_model.is_ok(), "Model optimization failed for {:?}", platform);

            let model_variant = optimized_model.unwrap();
            match platform {
                PlatformTarget::Mobile => {
                    assert!(model_variant.max_triangles <= 10000, "Mobile models should have triangle limits");
                    assert!(model_variant.lod_levels >= 3, "Mobile should use aggressive LOD");
                }
                PlatformTarget::Web => {
                    assert!(model_variant.compression.is_some(), "Web models should be compressed");
                    assert!(model_variant.streaming_enabled, "Web models should support streaming");
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_asset_pipeline_performance() {
        let mut fixture = AssetTestFixture::new();

        // Performance test: Import multiple assets concurrently
        let asset_count = 50;
        let mut asset_paths = Vec::new();

        // Create test assets
        for i in 0..asset_count {
            asset_paths.push(fixture.create_test_texture(&format!("perf_test_{}", i), "png"));
        }

        // Measure import performance
        let start_time = Instant::now();
        let mut import_results = Vec::new();

        for path in &asset_paths {
            let result = fixture.pipeline.import_asset(path);
            import_results.push(result);
        }

        let import_duration = start_time.elapsed();
        let average_import_time = import_duration.as_millis() / asset_count as u128;

        // Performance assertions
        assert!(average_import_time < 100, "Average import time too slow: {}ms", average_import_time);
        assert!(import_results.iter().all(|r| r.is_ok()), "All imports should succeed");

        // Test concurrent import performance
        let start_time = Instant::now();
        let handles: Vec<_> = asset_paths.chunks(10)
            .map(|chunk| {
                let chunk = chunk.to_vec();
                let pipeline = fixture.pipeline.clone();
                std::thread::spawn(move || {
                    chunk.iter().map(|path| pipeline.import_asset(path)).collect::<Vec<_>>()
                })
            })
            .collect();

        let concurrent_results: Vec<_> = handles.into_iter()
            .map(|h| h.join().unwrap())
            .flatten()
            .collect();

        let concurrent_duration = start_time.elapsed();

        // Concurrent imports should be faster than sequential
        assert!(concurrent_duration < import_duration, "Concurrent imports should be faster");
        assert!(concurrent_results.iter().all(|r| r.is_ok()), "All concurrent imports should succeed");
    }

    #[test]
    fn test_asset_pipeline_error_handling() {
        let mut fixture = AssetTestFixture::new();

        // Test invalid file format
        let invalid_path = fixture.test_assets_path.join("invalid.xyz");
        std::fs::write(&invalid_path, b"invalid content").expect("Failed to create invalid file");

        let result = fixture.pipeline.import_asset(&invalid_path);
        assert!(result.is_err(), "Should reject invalid file format");
        assert!(matches!(result.unwrap_err(), AssetError::UnsupportedFormat(_)));

        // Test corrupted file
        let corrupted_path = fixture.test_assets_path.join("corrupted.png");
        std::fs::write(&corrupted_path, b"not a PNG file").expect("Failed to create corrupted file");

        let result = fixture.pipeline.import_asset(&corrupted_path);
        assert!(result.is_err(), "Should reject corrupted file");
        assert!(matches!(result.unwrap_err(), AssetError::CorruptedFile(_)));

        // Test missing file
        let missing_path = fixture.test_assets_path.join("missing.png");
        let result = fixture.pipeline.import_asset(&missing_path);
        assert!(result.is_err(), "Should handle missing file");
        assert!(matches!(result.unwrap_err(), AssetError::FileNotFound(_)));

        // Test insufficient permissions (if possible)
        // This test might be skipped on some systems
        #[cfg(unix)]
        {
            let restricted_path = fixture.test_assets_path.join("restricted.png");
            std::fs::write(&restricted_path, b"content").expect("Failed to create restricted file");

            // Remove read permissions
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&restricted_path).unwrap().permissions();
            perms.set_mode(0o000);
            std::fs::set_permissions(&restricted_path, perms).unwrap();

            let result = fixture.pipeline.import_asset(&restricted_path);
            // Should handle permission error gracefully
            if result.is_err() {
                assert!(matches!(result.unwrap_err(), AssetError::PermissionDenied(_)));
            }
        }

        // Test database connection errors
        fixture.database.simulate_connection_error();
        let valid_path = fixture.create_test_texture("db_error_test", "png");
        let result = fixture.pipeline.import_asset(&valid_path);
        assert!(result.is_err(), "Should handle database errors");
        assert!(matches!(result.unwrap_err(), AssetError::DatabaseError(_)));

        // Test memory limits
        fixture.pipeline.set_memory_limit(1024); // Very low limit
        let large_texture_path = fixture.test_assets_path.join("large.png");
        // Create a larger fake PNG (just repeat the header pattern)
        let large_data = vec![0u8; 10240]; // 10KB of data
        std::fs::write(&large_texture_path, large_data).expect("Failed to create large file");

        let result = fixture.pipeline.import_asset(&large_texture_path);
        // Should either succeed with streaming or fail gracefully
        if result.is_err() {
            assert!(matches!(result.unwrap_err(), AssetError::OutOfMemory(_)));
        }
    }

    #[test]
    fn test_asset_memory_management() {
        let mut fixture = AssetTestFixture::new();

        // Test memory usage tracking
        let initial_memory = fixture.pipeline.get_memory_usage();
        assert_eq!(initial_memory, 0, "Initial memory usage should be zero");

        // Import several assets and track memory
        let mut asset_ids = Vec::new();
        for i in 0..10 {
            let path = fixture.create_test_texture(&format!("memory_test_{}", i), "png");
            let asset_id = fixture.pipeline.import_asset(&path).expect("Failed to import asset");
            asset_ids.push(asset_id);
        }

        let after_import_memory = fixture.pipeline.get_memory_usage();
        assert!(after_import_memory > initial_memory, "Memory usage should increase after imports");

        // Test asset unloading
        for asset_id in &asset_ids[..5] {
            fixture.pipeline.unload_asset(*asset_id);
        }

        let after_unload_memory = fixture.pipeline.get_memory_usage();
        assert!(after_unload_memory < after_import_memory, "Memory usage should decrease after unloading");

        // Test memory pressure handling
        fixture.pipeline.set_memory_pressure_threshold(after_unload_memory + 1000);

        // Add more assets to trigger memory pressure
        for i in 10..20 {
            let path = fixture.create_test_texture(&format!("pressure_test_{}", i), "png");
            let _ = fixture.pipeline.import_asset(&path);
        }

        // System should automatically unload least-recently-used assets
        let final_memory = fixture.pipeline.get_memory_usage();
        assert!(final_memory <= fixture.pipeline.get_memory_pressure_threshold() * 2);
    }
}

/// Mock types and implementations for testing
#[cfg(test)]
mod mocks {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    pub enum AssetType {
        Texture,
        Model,
        Audio,
        Material,
        Animation,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum AudioFormat {
        WAV,
        OGG,
        MP3,
        FLAC,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum TextureFormat {
        PNG,
        JPEG,
        TGA,
        WebP,
        DDS,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum PlatformTarget {
        Desktop,
        Mobile,
        Web,
        Console,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum QualityLevel {
        Low,
        Medium,
        High,
        Ultra,
    }

    #[derive(Debug)]
    pub enum AssetError {
        UnsupportedFormat(String),
        CorruptedFile(String),
        FileNotFound(String),
        PermissionDenied(String),
        DatabaseError(String),
        OutOfMemory(String),
    }
}