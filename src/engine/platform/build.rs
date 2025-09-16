/*!
 * Robin Engine Cross-Platform Build System
 * 
 * Automated build system supporting multiple platforms, architectures,
 * and deployment targets with optimized compilation pipelines.
 */

use crate::engine::{
    error::{RobinError, RobinResult},
    platform::{Platform, PlatformConfig, BuildConfiguration, OptimizationLevel},
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Cross-platform build system
#[derive(Debug)]
pub struct BuildSystem {
    config: PlatformConfig,
    build_cache: BuildCache,
    toolchains: HashMap<Platform, Toolchain>,
    build_profiles: HashMap<String, BuildProfile>,
    dependency_manager: DependencyManager,
}

impl BuildSystem {
    pub fn new(config: &PlatformConfig) -> RobinResult<Self> {
        let mut build_system = Self {
            config: config.clone(),
            build_cache: BuildCache::new()?,
            toolchains: HashMap::new(),
            build_profiles: HashMap::new(),
            dependency_manager: DependencyManager::new()?,
        };

        // Initialize toolchains for target platforms
        for platform in &config.target_platforms {
            let toolchain = Toolchain::detect_for_platform(platform)?;
            build_system.toolchains.insert(platform.clone(), toolchain);
        }

        // Create default build profiles
        build_system.create_default_profiles()?;

        Ok(build_system)
    }

    /// Setup build environment for all target platforms
    pub fn setup_build_environment(&mut self) -> RobinResult<()> {
        // Install required dependencies
        self.dependency_manager.install_platform_dependencies(&self.config.target_platforms)?;
        
        // Verify toolchains
        for (platform, toolchain) in &self.toolchains {
            if !toolchain.verify()? {
                return Err(RobinError::BuildError(
                    format!("Toolchain verification failed for platform: {:?}", platform)
                ));
            }
        }

        // Create build directories
        self.create_build_directories()?;
        
        Ok(())
    }

    /// Build for specific target platform
    pub fn build_for_target(&mut self, target: BuildTarget) -> RobinResult<BuildResult> {
        println!("Building for target: {:?}", target);
        
        let toolchain = self.toolchains.get(&target.platform)
            .ok_or_else(|| RobinError::BuildError(format!("No toolchain for platform: {:?}", target.platform)))?;

        // Check build cache
        if let Some(cached_result) = self.build_cache.get(&target) {
            if !target.force_rebuild && cached_result.is_valid() {
                println!("Using cached build for target: {:?}", target);
                return Ok(cached_result);
            }
        }

        let mut build_steps = Vec::new();

        // 1. Pre-build steps
        build_steps.push(BuildStep::PreBuild(PreBuildStep {
            clean_previous: target.clean_build,
            generate_build_info: true,
            validate_dependencies: true,
        }));

        // 2. Configure step
        build_steps.push(BuildStep::Configure(ConfigureStep {
            platform: target.platform.clone(),
            configuration: target.configuration.clone(),
            optimization: target.optimization.clone(),
            features: target.features.clone(),
            cross_compile: target.cross_compile,
        }));

        // 3. Compile step
        build_steps.push(BuildStep::Compile(CompileStep {
            parallel_jobs: target.parallel_jobs.unwrap_or(num_cpus::get() as u32),
            enable_lto: target.optimization == OptimizationLevel::Aggressive,
            enable_debug_symbols: target.configuration == BuildConfiguration::Debug || 
                                   target.configuration == BuildConfiguration::RelWithDebInfo,
        }));

        // 4. Link step
        build_steps.push(BuildStep::Link(LinkStep {
            static_linking: target.static_linking,
            strip_symbols: target.configuration == BuildConfiguration::MinSizeRel,
            optimize_size: target.configuration == BuildConfiguration::MinSizeRel,
        }));

        // 5. Post-build steps
        build_steps.push(BuildStep::PostBuild(PostBuildStep {
            run_tests: target.run_tests,
            package_artifacts: true,
            sign_binaries: target.sign_binaries,
            generate_symbols: target.configuration == BuildConfiguration::Debug,
        }));

        // Execute build steps
        let mut build_context = BuildContext {
            target: target.clone(),
            toolchain: toolchain.clone(),
            workspace: self.get_workspace_path(&target)?,
            output_dir: self.get_output_path(&target)?,
            intermediate_dir: self.get_intermediate_path(&target)?,
        };

        let start_time = std::time::Instant::now();
        
        for step in build_steps {
            self.execute_build_step(step, &mut build_context)?;
        }

        let build_duration = start_time.elapsed();

        let result = BuildResult {
            target: target.clone(),
            success: true,
            build_time: build_duration,
            output_files: self.collect_output_files(&build_context)?,
            warnings: Vec::new(),
            errors: Vec::new(),
            artifacts: self.collect_build_artifacts(&build_context)?,
            cache_key: self.calculate_cache_key(&target),
        };

        // Cache the build result
        self.build_cache.store(&target, &result)?;

        Ok(result)
    }

    /// Build all configured targets
    pub fn build_all_targets(&mut self) -> RobinResult<Vec<BuildResult>> {
        let mut results = Vec::new();
        
        for platform in &self.config.target_platforms.clone() {
            let target = BuildTarget::default_for_platform(platform);
            let result = self.build_for_target(target)?;
            results.push(result);
        }
        
        Ok(results)
    }

    /// Update configuration
    pub fn update_config(&mut self, config: &PlatformConfig) -> RobinResult<()> {
        self.config = config.clone();
        
        // Update toolchains for new platforms
        for platform in &config.target_platforms {
            if !self.toolchains.contains_key(platform) {
                let toolchain = Toolchain::detect_for_platform(platform)?;
                self.toolchains.insert(platform.clone(), toolchain);
            }
        }
        
        Ok(())
    }

    fn create_default_profiles(&mut self) -> RobinResult<()> {
        // Development profile
        let dev_profile = BuildProfile {
            name: "development".to_string(),
            configuration: BuildConfiguration::Debug,
            optimization: OptimizationLevel::None,
            enable_asserts: true,
            enable_logging: true,
            enable_profiling: true,
            parallel_jobs: Some(num_cpus::get() as u32),
            custom_flags: vec!["-g".to_string(), "-O0".to_string()],
        };
        self.build_profiles.insert("development".to_string(), dev_profile);

        // Release profile
        let release_profile = BuildProfile {
            name: "release".to_string(),
            configuration: BuildConfiguration::Release,
            optimization: OptimizationLevel::High,
            enable_asserts: false,
            enable_logging: false,
            enable_profiling: false,
            parallel_jobs: Some(num_cpus::get() as u32),
            custom_flags: vec!["-O3".to_string(), "-DNDEBUG".to_string()],
        };
        self.build_profiles.insert("release".to_string(), release_profile);

        // Distribution profile
        let dist_profile = BuildProfile {
            name: "distribution".to_string(),
            configuration: BuildConfiguration::MinSizeRel,
            optimization: OptimizationLevel::Aggressive,
            enable_asserts: false,
            enable_logging: false,
            enable_profiling: false,
            parallel_jobs: Some(num_cpus::get() as u32),
            custom_flags: vec!["-Os".to_string(), "-flto".to_string(), "-s".to_string()],
        };
        self.build_profiles.insert("distribution".to_string(), dist_profile);

        Ok(())
    }

    fn create_build_directories(&self) -> RobinResult<()> {
        let build_root = PathBuf::from("target/robin");
        
        if !build_root.exists() {
            std::fs::create_dir_all(&build_root)
                .map_err(|e| RobinError::IoError(format!("Failed to create build directory: {}", e)))?;
        }

        for platform in &self.config.target_platforms {
            let platform_dir = build_root.join(format!("{:?}", platform).to_lowercase());
            std::fs::create_dir_all(&platform_dir)
                .map_err(|e| RobinError::IoError(format!("Failed to create platform directory: {}", e)))?;
        }

        Ok(())
    }

    fn execute_build_step(&self, step: BuildStep, context: &mut BuildContext) -> RobinResult<()> {
        match step {
            BuildStep::PreBuild(pre_build) => {
                if pre_build.clean_previous {
                    self.clean_build_directory(&context.output_dir)?;
                }
                if pre_build.generate_build_info {
                    self.generate_build_info_file(context)?;
                }
                if pre_build.validate_dependencies {
                    self.validate_dependencies(context)?;
                }
            }
            BuildStep::Configure(configure) => {
                self.configure_build(configure, context)?;
            }
            BuildStep::Compile(compile) => {
                self.compile_sources(compile, context)?;
            }
            BuildStep::Link(link) => {
                self.link_executable(link, context)?;
            }
            BuildStep::PostBuild(post_build) => {
                if post_build.run_tests {
                    self.run_tests(context)?;
                }
                if post_build.package_artifacts {
                    self.package_artifacts(context)?;
                }
                if post_build.sign_binaries {
                    self.sign_binaries(context)?;
                }
            }
        }
        Ok(())
    }

    fn clean_build_directory(&self, dir: &Path) -> RobinResult<()> {
        if dir.exists() {
            std::fs::remove_dir_all(dir)
                .map_err(|e| RobinError::IoError(format!("Failed to clean build directory: {}", e)))?;
        }
        std::fs::create_dir_all(dir)
            .map_err(|e| RobinError::IoError(format!("Failed to create clean build directory: {}", e)))?;
        Ok(())
    }

    fn generate_build_info_file(&self, context: &BuildContext) -> RobinResult<()> {
        let build_info = BuildInfo {
            timestamp: chrono::Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            target_platform: context.target.platform.clone(),
            configuration: context.target.configuration.clone(),
            optimization: context.target.optimization.clone(),
            git_commit: self.get_git_commit_hash().unwrap_or_else(|_| "unknown".to_string()),
            compiler_version: context.toolchain.get_compiler_version()?,
        };

        let build_info_path = context.intermediate_dir.join("build_info.json");
        let build_info_json = serde_json::to_string_pretty(&build_info)
            .map_err(|e| RobinError::SerializationError { 
                object_type: "BuildInfo".to_string(), 
                reason: format!("Failed to serialize build info: {}", e) 
            })?;
        
        std::fs::write(build_info_path, build_info_json)
            .map_err(|e| RobinError::IoError(format!("Failed to write build info: {}", e)))?;

        Ok(())
    }

    fn validate_dependencies(&self, context: &BuildContext) -> RobinResult<()> {
        // Validate that all required dependencies are available
        context.toolchain.validate_dependencies()?;
        Ok(())
    }

    fn configure_build(&self, configure: ConfigureStep, context: &BuildContext) -> RobinResult<()> {
        // Generate platform-specific configuration
        let cargo_args = self.generate_cargo_args(&configure)?;
        
        let mut cmd = Command::new("cargo");
        cmd.args(&cargo_args);
        cmd.current_dir(&context.workspace);
        
        let output = cmd.output()
            .map_err(|e| RobinError::BuildError(format!("Failed to run cargo configure: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RobinError::BuildError(format!("Configure failed: {}", stderr)));
        }

        Ok(())
    }

    fn compile_sources(&self, compile: CompileStep, context: &BuildContext) -> RobinResult<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("build");
        
        if compile.parallel_jobs > 1 {
            cmd.arg("--jobs").arg(compile.parallel_jobs.to_string());
        }
        
        cmd.current_dir(&context.workspace);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        println!("Compiling sources...");
        let output = cmd.output()
            .map_err(|e| RobinError::BuildError(format!("Failed to compile: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RobinError::BuildError(format!("Compilation failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Compilation output: {}", stdout);

        Ok(())
    }

    fn link_executable(&self, link: LinkStep, context: &BuildContext) -> RobinResult<()> {
        // Linking is typically handled by cargo, but we can add post-link processing here
        if link.strip_symbols {
            self.strip_debug_symbols(context)?;
        }
        
        Ok(())
    }

    fn run_tests(&self, context: &BuildContext) -> RobinResult<()> {
        let mut cmd = Command::new("cargo");
        cmd.arg("test");
        cmd.current_dir(&context.workspace);
        
        println!("Running tests...");
        let output = cmd.output()
            .map_err(|e| RobinError::BuildError(format!("Failed to run tests: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(RobinError::BuildError(format!("Tests failed: {}", stderr)));
        }

        Ok(())
    }

    fn package_artifacts(&self, context: &BuildContext) -> RobinResult<()> {
        // Create package directory
        let package_dir = context.output_dir.join("package");
        std::fs::create_dir_all(&package_dir)
            .map_err(|e| RobinError::IoError(format!("Failed to create package directory: {}", e)))?;

        // Copy artifacts based on platform
        match context.target.platform {
            Platform::Windows => {
                self.package_windows_artifacts(context, &package_dir)?;
            }
            Platform::MacOS => {
                self.package_macos_artifacts(context, &package_dir)?;
            }
            Platform::Linux => {
                self.package_linux_artifacts(context, &package_dir)?;
            }
            Platform::iOS => {
                self.package_ios_artifacts(context, &package_dir)?;
            }
            Platform::Android => {
                self.package_android_artifacts(context, &package_dir)?;
            }
            Platform::Web => {
                self.package_web_artifacts(context, &package_dir)?;
            }
        }

        Ok(())
    }

    fn sign_binaries(&self, context: &BuildContext) -> RobinResult<()> {
        // Code signing would be implemented here based on platform requirements
        println!("Code signing not implemented for platform: {:?}", context.target.platform);
        Ok(())
    }

    // Platform-specific packaging methods
    fn package_windows_artifacts(&self, context: &BuildContext, package_dir: &Path) -> RobinResult<()> {
        // Copy executable and DLLs
        let exe_name = format!("robin{}", context.target.platform.get_executable_extension());
        let src_exe = context.output_dir.join(&exe_name);
        let dst_exe = package_dir.join(&exe_name);
        
        if src_exe.exists() {
            std::fs::copy(&src_exe, &dst_exe)
                .map_err(|e| RobinError::IoError(format!("Failed to copy executable: {}", e)))?;
        }
        
        Ok(())
    }

    fn package_macos_artifacts(&self, context: &BuildContext, package_dir: &Path) -> RobinResult<()> {
        // Create .app bundle structure
        let app_name = "Robin.app";
        let app_dir = package_dir.join(app_name);
        let contents_dir = app_dir.join("Contents");
        let macos_dir = contents_dir.join("MacOS");
        let resources_dir = contents_dir.join("Resources");

        std::fs::create_dir_all(&macos_dir)
            .map_err(|e| RobinError::IoError(format!("Failed to create MacOS directory: {}", e)))?;
        std::fs::create_dir_all(&resources_dir)
            .map_err(|e| RobinError::IoError(format!("Failed to create Resources directory: {}", e)))?;

        // Copy executable
        let src_exe = context.output_dir.join("robin");
        let dst_exe = macos_dir.join("robin");
        
        if src_exe.exists() {
            std::fs::copy(&src_exe, &dst_exe)
                .map_err(|e| RobinError::IoError(format!("Failed to copy executable: {}", e)))?;
        }

        // Create Info.plist
        self.create_macos_info_plist(&contents_dir)?;
        
        Ok(())
    }

    fn package_linux_artifacts(&self, context: &BuildContext, package_dir: &Path) -> RobinResult<()> {
        // Copy executable and shared libraries
        let exe_name = "robin";
        let src_exe = context.output_dir.join(exe_name);
        let dst_exe = package_dir.join(exe_name);
        
        if src_exe.exists() {
            std::fs::copy(&src_exe, &dst_exe)
                .map_err(|e| RobinError::IoError(format!("Failed to copy executable: {}", e)))?;
        }
        
        Ok(())
    }

    fn package_ios_artifacts(&self, context: &BuildContext, package_dir: &Path) -> RobinResult<()> {
        // iOS packaging would create IPA file
        println!("iOS packaging not fully implemented");
        Ok(())
    }

    fn package_android_artifacts(&self, context: &BuildContext, package_dir: &Path) -> RobinResult<()> {
        // Android packaging would create APK file
        println!("Android packaging not fully implemented");
        Ok(())
    }

    fn package_web_artifacts(&self, context: &BuildContext, package_dir: &Path) -> RobinResult<()> {
        // Copy WASM and JS files
        let wasm_name = "robin.wasm";
        let js_name = "robin.js";
        
        let src_wasm = context.output_dir.join(wasm_name);
        let src_js = context.output_dir.join(js_name);
        let dst_wasm = package_dir.join(wasm_name);
        let dst_js = package_dir.join(js_name);
        
        if src_wasm.exists() {
            std::fs::copy(&src_wasm, &dst_wasm)
                .map_err(|e| RobinError::IoError(format!("Failed to copy WASM: {}", e)))?;
        }
        
        if src_js.exists() {
            std::fs::copy(&src_js, &dst_js)
                .map_err(|e| RobinError::IoError(format!("Failed to copy JS: {}", e)))?;
        }
        
        Ok(())
    }

    fn create_macos_info_plist(&self, contents_dir: &Path) -> RobinResult<()> {
        let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>robin</string>
    <key>CFBundleIdentifier</key>
    <string>com.robin.engine</string>
    <key>CFBundleName</key>
    <string>Robin Engine</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>"#;

        let plist_path = contents_dir.join("Info.plist");
        std::fs::write(plist_path, info_plist)
            .map_err(|e| RobinError::IoError(format!("Failed to write Info.plist: {}", e)))?;
        
        Ok(())
    }

    fn strip_debug_symbols(&self, context: &BuildContext) -> RobinResult<()> {
        // Platform-specific symbol stripping
        match context.target.platform {
            Platform::Linux | Platform::MacOS => {
                let exe_path = context.output_dir.join("robin");
                if exe_path.exists() {
                    let mut cmd = Command::new("strip");
                    cmd.arg(&exe_path);
                    
                    let output = cmd.output()
                        .map_err(|e| RobinError::BuildError(format!("Failed to strip symbols: {}", e)))?;
                    
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        return Err(RobinError::BuildError(format!("Strip failed: {}", stderr)));
                    }
                }
            }
            _ => {
                // Other platforms don't use strip or have different methods
            }
        }
        
        Ok(())
    }

    fn generate_cargo_args(&self, configure: &ConfigureStep) -> RobinResult<Vec<String>> {
        let mut args = vec!["build".to_string()];
        
        match configure.configuration {
            BuildConfiguration::Release | BuildConfiguration::RelWithDebInfo | BuildConfiguration::MinSizeRel => {
                args.push("--release".to_string());
            }
            _ => {}
        }
        
        if configure.cross_compile {
            args.push("--target".to_string());
            args.push(self.get_target_triple(&configure.platform)?);
        }
        
        // Add feature flags
        if !configure.features.is_empty() {
            args.push("--features".to_string());
            args.push(configure.features.join(","));
        }
        
        Ok(args)
    }

    fn get_target_triple(&self, platform: &Platform) -> RobinResult<String> {
        let triple = match platform {
            Platform::Windows => "x86_64-pc-windows-msvc",
            Platform::MacOS => "x86_64-apple-darwin",
            Platform::Linux => "x86_64-unknown-linux-gnu",
            Platform::iOS => "aarch64-apple-ios",
            Platform::Android => "aarch64-linux-android",
            Platform::Web => "wasm32-unknown-unknown",
        };
        
        Ok(triple.to_string())
    }

    fn get_workspace_path(&self, target: &BuildTarget) -> RobinResult<PathBuf> {
        Ok(PathBuf::from("."))
    }

    fn get_output_path(&self, target: &BuildTarget) -> RobinResult<PathBuf> {
        let mut path = PathBuf::from("target");
        
        if target.cross_compile {
            path.push(self.get_target_triple(&target.platform)?);
        }
        
        match target.configuration {
            BuildConfiguration::Debug => path.push("debug"),
            _ => path.push("release"),
        }
        
        Ok(path)
    }

    fn get_intermediate_path(&self, target: &BuildTarget) -> RobinResult<PathBuf> {
        let output_path = self.get_output_path(target)?;
        Ok(output_path.join("intermediate"))
    }

    fn collect_output_files(&self, context: &BuildContext) -> RobinResult<Vec<PathBuf>> {
        let mut files = Vec::new();
        
        if let Ok(entries) = std::fs::read_dir(&context.output_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                    files.push(entry.path());
                }
            }
        }
        
        Ok(files)
    }

    fn collect_build_artifacts(&self, context: &BuildContext) -> RobinResult<Vec<BuildArtifact>> {
        let mut artifacts = Vec::new();
        
        // Collect executable
        let exe_name = format!("robin{}", context.target.platform.get_executable_extension());
        let exe_path = context.output_dir.join(&exe_name);
        
        if exe_path.exists() {
            artifacts.push(BuildArtifact {
                name: exe_name,
                path: exe_path,
                artifact_type: ArtifactType::Executable,
                size: 0, // Would be calculated
            });
        }
        
        Ok(artifacts)
    }

    fn calculate_cache_key(&self, target: &BuildTarget) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        target.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    fn get_git_commit_hash(&self) -> RobinResult<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .map_err(|e| RobinError::BuildError(format!("Failed to get git commit: {}", e)))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(RobinError::BuildError("Failed to get git commit hash".to_string()))
        }
    }
}

/// Build target specification
#[derive(Debug, Clone, Hash)]
pub struct BuildTarget {
    pub platform: Platform,
    pub configuration: BuildConfiguration,
    pub optimization: OptimizationLevel,
    pub features: Vec<String>,
    pub cross_compile: bool,
    pub parallel_jobs: Option<u32>,
    pub static_linking: bool,
    pub force_rebuild: bool,
    pub clean_build: bool,
    pub run_tests: bool,
    pub sign_binaries: bool,
}

impl BuildTarget {
    pub fn default_for_platform(platform: &Platform) -> Self {
        Self {
            platform: platform.clone(),
            configuration: BuildConfiguration::Release,
            optimization: OptimizationLevel::High,
            features: Vec::new(),
            cross_compile: false,
            parallel_jobs: None,
            static_linking: false,
            force_rebuild: false,
            clean_build: false,
            run_tests: false,
            sign_binaries: platform == &Platform::MacOS || platform == &Platform::iOS,
        }
    }
}

/// Build result information
#[derive(Debug, Clone)]
pub struct BuildResult {
    pub target: BuildTarget,
    pub success: bool,
    pub build_time: std::time::Duration,
    pub output_files: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub artifacts: Vec<BuildArtifact>,
    pub cache_key: String,
}

impl BuildResult {
    pub fn is_valid(&self) -> bool {
        self.success && self.artifacts.iter().all(|a| a.path.exists())
    }
}

/// Build artifact information
#[derive(Debug, Clone)]
pub struct BuildArtifact {
    pub name: String,
    pub path: PathBuf,
    pub artifact_type: ArtifactType,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    Executable,
    Library,
    Archive,
    Package,
    Symbols,
}

/// Platform-specific toolchain
#[derive(Debug, Clone)]
pub struct Toolchain {
    pub platform: Platform,
    pub compiler: String,
    pub linker: String,
    pub archiver: String,
    pub debugger: String,
    pub package_manager: Option<String>,
    pub sdk_path: Option<PathBuf>,
    pub version: String,
}

impl Toolchain {
    pub fn detect_for_platform(platform: &Platform) -> RobinResult<Self> {
        match platform {
            Platform::Windows => Ok(Self {
                platform: platform.clone(),
                compiler: "rustc".to_string(),
                linker: "link.exe".to_string(),
                archiver: "lib.exe".to_string(),
                debugger: "windbg".to_string(),
                package_manager: Some("cargo".to_string()),
                sdk_path: None,
                version: "stable".to_string(),
            }),
            Platform::MacOS => Ok(Self {
                platform: platform.clone(),
                compiler: "rustc".to_string(),
                linker: "ld".to_string(),
                archiver: "ar".to_string(),
                debugger: "lldb".to_string(),
                package_manager: Some("cargo".to_string()),
                sdk_path: None,
                version: "stable".to_string(),
            }),
            Platform::Linux => Ok(Self {
                platform: platform.clone(),
                compiler: "rustc".to_string(),
                linker: "ld".to_string(),
                archiver: "ar".to_string(),
                debugger: "gdb".to_string(),
                package_manager: Some("cargo".to_string()),
                sdk_path: None,
                version: "stable".to_string(),
            }),
            _ => Ok(Self {
                platform: platform.clone(),
                compiler: "rustc".to_string(),
                linker: "ld".to_string(),
                archiver: "ar".to_string(),
                debugger: "gdb".to_string(),
                package_manager: Some("cargo".to_string()),
                sdk_path: None,
                version: "stable".to_string(),
            }),
        }
    }

    pub fn verify(&self) -> RobinResult<bool> {
        // Verify that required tools are available
        let tools = [&self.compiler, &self.linker, &self.archiver];
        
        for tool in tools {
            if which::which(tool).is_err() {
                return Ok(false);
            }
        }
        
        Ok(true)
    }

    pub fn get_compiler_version(&self) -> RobinResult<String> {
        let output = Command::new(&self.compiler)
            .arg("--version")
            .output()
            .map_err(|e| RobinError::BuildError(format!("Failed to get compiler version: {}", e)))?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).lines().next().unwrap_or("unknown").to_string())
        } else {
            Ok("unknown".to_string())
        }
    }

    pub fn validate_dependencies(&self) -> RobinResult<()> {
        // Validate that all required dependencies are available
        if let Some(package_manager) = &self.package_manager {
            let output = Command::new(package_manager)
                .arg("--version")
                .output()
                .map_err(|e| RobinError::BuildError(format!("Package manager validation failed: {}", e)))?;
            
            if !output.status.success() {
                return Err(RobinError::BuildError("Package manager not available".to_string()));
            }
        }
        
        Ok(())
    }
}

/// Build cache for avoiding redundant builds
#[derive(Debug)]
pub struct BuildCache {
    cache_dir: PathBuf,
    entries: HashMap<String, BuildResult>,
}

impl BuildCache {
    pub fn new() -> RobinResult<Self> {
        let cache_dir = PathBuf::from("target/robin/cache");
        std::fs::create_dir_all(&cache_dir)
            .map_err(|e| RobinError::IoError(format!("Failed to create cache directory: {}", e)))?;
        
        Ok(Self {
            cache_dir,
            entries: HashMap::new(),
        })
    }

    pub fn get(&self, target: &BuildTarget) -> Option<BuildResult> {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        target.hash(&mut hasher);
        let key = format!("{:x}", hasher.finish());
        
        self.entries.get(&key).cloned()
    }

    pub fn store(&mut self, target: &BuildTarget, result: &BuildResult) -> RobinResult<()> {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        target.hash(&mut hasher);
        let key = format!("{:x}", hasher.finish());
        
        self.entries.insert(key, result.clone());
        Ok(())
    }
}

/// Dependency manager for platform-specific dependencies
#[derive(Debug)]
pub struct DependencyManager {
    platform_deps: HashMap<Platform, Vec<Dependency>>,
}

impl DependencyManager {
    pub fn new() -> RobinResult<Self> {
        let mut platform_deps = HashMap::new();
        
        // Windows dependencies
        platform_deps.insert(Platform::Windows, vec![
            Dependency {
                name: "Visual Studio Build Tools".to_string(),
                version: "2019".to_string(),
                required: true,
                install_command: None,
            },
        ]);
        
        // macOS dependencies
        platform_deps.insert(Platform::MacOS, vec![
            Dependency {
                name: "Xcode Command Line Tools".to_string(),
                version: "latest".to_string(),
                required: true,
                install_command: Some("xcode-select --install".to_string()),
            },
        ]);
        
        Ok(Self { platform_deps })
    }

    pub fn install_platform_dependencies(&self, platforms: &[Platform]) -> RobinResult<()> {
        for platform in platforms {
            if let Some(deps) = self.platform_deps.get(platform) {
                for dep in deps {
                    if dep.required && !self.is_dependency_installed(dep)? {
                        self.install_dependency(dep)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn is_dependency_installed(&self, dep: &Dependency) -> RobinResult<bool> {
        // Check if dependency is installed (simplified)
        Ok(true) // Placeholder
    }

    fn install_dependency(&self, dep: &Dependency) -> RobinResult<()> {
        if let Some(install_cmd) = &dep.install_command {
            println!("Installing dependency: {} using command: {}", dep.name, install_cmd);
            // Would execute installation command here
        } else {
            println!("Please manually install dependency: {} (version: {})", dep.name, dep.version);
        }
        Ok(())
    }
}

/// Platform dependency specification
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub required: bool,
    pub install_command: Option<String>,
}

/// Build profile for common configurations
#[derive(Debug, Clone)]
pub struct BuildProfile {
    pub name: String,
    pub configuration: BuildConfiguration,
    pub optimization: OptimizationLevel,
    pub enable_asserts: bool,
    pub enable_logging: bool,
    pub enable_profiling: bool,
    pub parallel_jobs: Option<u32>,
    pub custom_flags: Vec<String>,
}

/// Build steps
#[derive(Debug)]
pub enum BuildStep {
    PreBuild(PreBuildStep),
    Configure(ConfigureStep),
    Compile(CompileStep),
    Link(LinkStep),
    PostBuild(PostBuildStep),
}

#[derive(Debug)]
pub struct PreBuildStep {
    pub clean_previous: bool,
    pub generate_build_info: bool,
    pub validate_dependencies: bool,
}

#[derive(Debug)]
pub struct ConfigureStep {
    pub platform: Platform,
    pub configuration: BuildConfiguration,
    pub optimization: OptimizationLevel,
    pub features: Vec<String>,
    pub cross_compile: bool,
}

#[derive(Debug)]
pub struct CompileStep {
    pub parallel_jobs: u32,
    pub enable_lto: bool,
    pub enable_debug_symbols: bool,
}

#[derive(Debug)]
pub struct LinkStep {
    pub static_linking: bool,
    pub strip_symbols: bool,
    pub optimize_size: bool,
}

#[derive(Debug)]
pub struct PostBuildStep {
    pub run_tests: bool,
    pub package_artifacts: bool,
    pub sign_binaries: bool,
    pub generate_symbols: bool,
}

/// Build context passed through build steps
#[derive(Debug)]
pub struct BuildContext {
    pub target: BuildTarget,
    pub toolchain: Toolchain,
    pub workspace: PathBuf,
    pub output_dir: PathBuf,
    pub intermediate_dir: PathBuf,
}

/// Build information embedded in artifacts
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct BuildInfo {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub target_platform: Platform,
    pub configuration: BuildConfiguration,
    pub optimization: OptimizationLevel,
    pub git_commit: String,
    pub compiler_version: String,
}