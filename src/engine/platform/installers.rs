/// Cross-Platform Installer Generation for Robin Game Engine
///
/// Creates native installers for different platforms and distribution methods

use crate::engine::core::RobinResult;
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use super::Platform;

/// Installer generator
pub struct InstallerGenerator {
    config: InstallerConfig,
    build_directory: PathBuf,
    output_directory: PathBuf,
    resources: HashMap<String, PathBuf>,
}

impl InstallerGenerator {
    /// Create new installer generator
    pub fn new(config: InstallerConfig) -> Self {
        let build_directory = PathBuf::from("build");
        let output_directory = PathBuf::from("dist");

        Self {
            config,
            build_directory,
            output_directory,
            resources: HashMap::new(),
        }
    }

    /// Generate installer for target platform
    pub fn generate(&mut self, platform: Platform) -> RobinResult<PathBuf> {
        println!("📦 Generating installer for {:?}", platform);

        // Prepare build directory
        self.prepare_build_directory()?;

        // Copy game files
        self.copy_game_files()?;

        // Copy resources
        self.copy_resources(platform)?;

        // Generate platform-specific installer
        let installer_path = match platform {
            Platform::Windows => self.generate_windows_installer()?,
            Platform::MacOS => self.generate_macos_installer()?,
            Platform::Linux => self.generate_linux_installer()?,
            Platform::Steam => self.generate_steam_package()?,
            _ => return Err("Unsupported platform for installer generation".into()),
        };

        println!("✅ Installer generated: {}", installer_path.display());
        Ok(installer_path)
    }

    /// Prepare build directory
    fn prepare_build_directory(&self) -> RobinResult<()> {
        // Clean and create build directory
        if self.build_directory.exists() {
            fs::remove_dir_all(&self.build_directory)?;
        }
        fs::create_dir_all(&self.build_directory)?;
        fs::create_dir_all(&self.output_directory)?;

        Ok(())
    }

    /// Copy game files to build directory
    fn copy_game_files(&self) -> RobinResult<()> {
        println!("📁 Copying game files");

        // Create directory structure
        let game_dir = self.build_directory.join("game");
        fs::create_dir_all(&game_dir)?;

        // Copy executable
        let exe_name = if cfg!(windows) {
            "robin.exe"
        } else {
            "robin"
        };

        let exe_path = PathBuf::from("target/release").join(exe_name);
        if exe_path.exists() {
            fs::copy(&exe_path, game_dir.join(exe_name))?;
        }

        // Copy data files
        let data_dir = game_dir.join("data");
        fs::create_dir_all(&data_dir)?;

        // Copy assets
        let assets_dir = game_dir.join("assets");
        fs::create_dir_all(&assets_dir)?;

        Ok(())
    }

    /// Copy platform-specific resources
    fn copy_resources(&mut self, platform: Platform) -> RobinResult<()> {
        println!("📋 Copying platform resources");

        match platform {
            Platform::Windows => {
                // Windows-specific resources
                self.resources.insert(
                    "icon".to_string(),
                    PathBuf::from("resources/windows/icon.ico")
                );
                self.resources.insert(
                    "license".to_string(),
                    PathBuf::from("LICENSE.txt")
                );
            }
            Platform::MacOS => {
                // macOS-specific resources
                self.resources.insert(
                    "icon".to_string(),
                    PathBuf::from("resources/macos/icon.icns")
                );
                self.resources.insert(
                    "info_plist".to_string(),
                    PathBuf::from("resources/macos/Info.plist")
                );
            }
            Platform::Linux => {
                // Linux-specific resources
                self.resources.insert(
                    "desktop".to_string(),
                    PathBuf::from("resources/linux/robin.desktop")
                );
                self.resources.insert(
                    "icon".to_string(),
                    PathBuf::from("resources/linux/icon.png")
                );
            }
            _ => {}
        }

        Ok(())
    }

    /// Generate Windows installer (NSIS or MSI)
    fn generate_windows_installer(&self) -> RobinResult<PathBuf> {
        println!("🪟 Generating Windows installer");

        let installer_path = self.output_directory.join(format!(
            "{}-{}-windows-installer.exe",
            self.config.app_name,
            self.config.version
        ));

        // Generate NSIS script
        let nsis_script = self.generate_nsis_script()?;
        let script_path = self.build_directory.join("installer.nsi");
        fs::write(&script_path, nsis_script)?;

        // In production, would run NSIS compiler
        println!("📝 NSIS script generated at: {}", script_path.display());

        // For now, simulate installer creation
        fs::write(&installer_path, "Windows Installer Placeholder")?;

        Ok(installer_path)
    }

    /// Generate NSIS installer script
    fn generate_nsis_script(&self) -> RobinResult<String> {
        let script = format!(r#"
; NSIS Installer Script for {}
!define APPNAME "{}"
!define COMPANYNAME "{}"
!define DESCRIPTION "{}"
!define VERSIONMAJOR {}
!define VERSIONMINOR {}
!define VERSIONPATCH {}
!define INSTALLSIZE 100000

RequestExecutionLevel admin
InstallDir "$PROGRAMFILES\${{APPNAME}}"

Name "${{APPNAME}}"
OutFile "{}-installer.exe"

!include LogicLib.nsh
!include MUI2.nsh

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE.txt"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

Section "install"
    SetOutPath "$INSTDIR"

    File /r "game\*.*"

    WriteUninstaller "$INSTDIR\uninstall.exe"

    # Registry information
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{APPNAME}}" "DisplayName" "${{APPNAME}}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{APPNAME}}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{APPNAME}}" "DisplayIcon" "$INSTDIR\robin.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{APPNAME}}" "Publisher" "${{COMPANYNAME}}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{APPNAME}}" "DisplayVersion" "${{VERSIONMAJOR}}.${{VERSIONMINOR}}.${{VERSIONPATCH}}"
    WriteRegDWORD HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{APPNAME}}" "EstimatedSize" ${{INSTALLSIZE}}

    # Create shortcuts
    CreateDirectory "$SMPROGRAMS\${{APPNAME}}"
    CreateShortcut "$SMPROGRAMS\${{APPNAME}}\${{APPNAME}}.lnk" "$INSTDIR\robin.exe"
    CreateShortcut "$DESKTOP\${{APPNAME}}.lnk" "$INSTDIR\robin.exe"
SectionEnd

Section "uninstall"
    Delete "$INSTDIR\*.*"
    Delete "$DESKTOP\${{APPNAME}}.lnk"
    Delete "$SMPROGRAMS\${{APPNAME}}\*.*"
    RmDir "$SMPROGRAMS\${{APPNAME}}"
    RmDir /r "$INSTDIR"

    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${{APPNAME}}"
SectionEnd
"#,
            self.config.app_name,
            self.config.app_name,
            self.config.company_name,
            self.config.description,
            self.config.version_major,
            self.config.version_minor,
            self.config.version_patch,
            self.config.app_name
        );

        Ok(script)
    }

    /// Generate macOS installer (.app bundle and DMG)
    fn generate_macos_installer(&self) -> RobinResult<PathBuf> {
        println!("🍎 Generating macOS installer");

        // Create app bundle structure
        let app_name = format!("{}.app", self.config.app_name);
        let app_bundle = self.build_directory.join(&app_name);
        let contents = app_bundle.join("Contents");
        let macos_dir = contents.join("MacOS");
        let resources_dir = contents.join("Resources");

        fs::create_dir_all(&macos_dir)?;
        fs::create_dir_all(&resources_dir)?;

        // Copy executable
        let exe_path = self.build_directory.join("game/robin");
        if exe_path.exists() {
            fs::copy(&exe_path, macos_dir.join(&self.config.app_name))?;
        }

        // Create Info.plist
        let info_plist = self.generate_info_plist()?;
        fs::write(contents.join("Info.plist"), info_plist)?;

        // Copy icon
        if let Some(icon_path) = self.resources.get("icon") {
            if icon_path.exists() {
                fs::copy(icon_path, resources_dir.join("icon.icns"))?;
            }
        }

        // Create DMG
        let dmg_path = self.output_directory.join(format!(
            "{}-{}-macos.dmg",
            self.config.app_name,
            self.config.version
        ));

        // In production, would run hdiutil to create DMG
        println!("📦 DMG would be created at: {}", dmg_path.display());

        // For now, simulate DMG creation
        fs::write(&dmg_path, "macOS DMG Placeholder")?;

        Ok(dmg_path)
    }

    /// Generate Info.plist for macOS
    fn generate_info_plist(&self) -> RobinResult<String> {
        let plist = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIconFile</key>
    <string>icon.icns</string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>{}.{}.{}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.12</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>"#,
            self.config.app_name,
            self.config.bundle_identifier,
            self.config.app_name,
            self.config.version,
            self.config.version_major,
            self.config.version_minor,
            self.config.version_patch
        );

        Ok(plist)
    }

    /// Generate Linux installer (AppImage, DEB, or RPM)
    fn generate_linux_installer(&self) -> RobinResult<PathBuf> {
        println!("🐧 Generating Linux installer");

        // We'll generate an AppImage for maximum compatibility
        let appimage_path = self.output_directory.join(format!(
            "{}-{}-linux-x86_64.AppImage",
            self.config.app_name,
            self.config.version
        ));

        // Create AppDir structure
        let app_dir = self.build_directory.join("AppDir");
        fs::create_dir_all(&app_dir)?;
        fs::create_dir_all(app_dir.join("usr/bin"))?;
        fs::create_dir_all(app_dir.join("usr/share/applications"))?;
        fs::create_dir_all(app_dir.join("usr/share/icons"))?;

        // Copy executable
        let exe_path = self.build_directory.join("game/robin");
        if exe_path.exists() {
            fs::copy(&exe_path, app_dir.join("usr/bin/robin"))?;
        }

        // Create desktop file
        let desktop_file = self.generate_desktop_file()?;
        fs::write(
            app_dir.join("usr/share/applications/robin.desktop"),
            desktop_file
        )?;

        // Copy icon
        if let Some(icon_path) = self.resources.get("icon") {
            if icon_path.exists() {
                fs::copy(icon_path, app_dir.join("usr/share/icons/robin.png"))?;
            }
        }

        // Create AppRun script
        let apprun = r#"#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/robin" "$@"
"#;
        let apprun_path = app_dir.join("AppRun");
        fs::write(&apprun_path, apprun)?;

        // Make AppRun executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&apprun_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&apprun_path, perms)?;
        }

        // In production, would run appimagetool
        println!("📦 AppImage would be created at: {}", appimage_path.display());

        // For now, simulate AppImage creation
        fs::write(&appimage_path, "Linux AppImage Placeholder")?;

        Ok(appimage_path)
    }

    /// Generate .desktop file for Linux
    fn generate_desktop_file(&self) -> RobinResult<String> {
        let desktop = format!(r#"[Desktop Entry]
Type=Application
Name={}
Comment={}
Exec=robin %F
Icon=robin
Categories=Game;
Terminal=false
StartupNotify=true
"#,
            self.config.app_name,
            self.config.description
        );

        Ok(desktop)
    }

    /// Generate Steam package
    fn generate_steam_package(&self) -> RobinResult<PathBuf> {
        println!("🎮 Generating Steam package");

        let steam_dir = self.build_directory.join("steam");
        fs::create_dir_all(&steam_dir)?;

        // Copy game files
        self.copy_game_files_to(&steam_dir)?;

        // Generate Steam build scripts
        self.generate_steam_build_scripts(&steam_dir)?;

        let package_path = self.output_directory.join(format!(
            "{}-{}-steam.zip",
            self.config.app_name,
            self.config.version
        ));

        // In production, would create ZIP archive
        println!("📦 Steam package would be created at: {}", package_path.display());

        // For now, simulate package creation
        fs::write(&package_path, "Steam Package Placeholder")?;

        Ok(package_path)
    }

    /// Copy game files to specific directory
    fn copy_game_files_to(&self, target: &Path) -> RobinResult<()> {
        // Implementation would copy all game files to target directory
        fs::create_dir_all(target)?;
        Ok(())
    }

    /// Generate Steam build scripts
    fn generate_steam_build_scripts(&self, steam_dir: &Path) -> RobinResult<()> {
        // app_build script
        let app_build = format!(r#"
"AppBuild"
{{
    "AppID" "{}"
    "Desc" "{}"
    "ContentRoot" "."
    "BuildOutput" "../output/"

    "Depots"
    {{
        "{}" // Main depot
        {{
            "FileMapping"
            {{
                "LocalPath" "*"
                "DepotPath" "."
                "Recursive" "1"
            }}
        }}
    }}
}}
"#,
            self.config.steam_app_id.unwrap_or(0),
            self.config.description,
            self.config.steam_app_id.unwrap_or(0) + 1
        );

        fs::write(steam_dir.join("app_build.vdf"), app_build)?;
        Ok(())
    }
}

/// Installer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallerConfig {
    pub app_name: String,
    pub display_name: String,
    pub version: String,
    pub version_major: u32,
    pub version_minor: u32,
    pub version_patch: u32,
    pub company_name: String,
    pub description: String,
    pub bundle_identifier: String,
    pub license_file: Option<PathBuf>,
    pub readme_file: Option<PathBuf>,
    pub icon_file: Option<PathBuf>,
    pub steam_app_id: Option<u32>,
    pub install_size_mb: u64,
    pub requires_admin: bool,
    pub auto_launch: bool,
    pub create_desktop_shortcut: bool,
    pub create_start_menu_shortcut: bool,
}

impl Default for InstallerConfig {
    fn default() -> Self {
        Self {
            app_name: "Robin".to_string(),
            display_name: "Robin Game Engine".to_string(),
            version: "1.0.0".to_string(),
            version_major: 1,
            version_minor: 0,
            version_patch: 0,
            company_name: "Robin Games".to_string(),
            description: "Advanced game engine with Engineer Build Mode".to_string(),
            bundle_identifier: "com.robingames.robin".to_string(),
            license_file: Some(PathBuf::from("LICENSE.txt")),
            readme_file: Some(PathBuf::from("README.md")),
            icon_file: None,
            steam_app_id: None,
            install_size_mb: 500,
            requires_admin: false,
            auto_launch: false,
            create_desktop_shortcut: true,
            create_start_menu_shortcut: true,
        }
    }
}

/// Package manager for distribution
pub struct PackageManager {
    packages: HashMap<Platform, PackageInfo>,
}

impl PackageManager {
    /// Create new package manager
    pub fn new() -> Self {
        Self {
            packages: HashMap::new(),
        }
    }

    /// Register package
    pub fn register_package(&mut self, platform: Platform, info: PackageInfo) {
        self.packages.insert(platform, info);
    }

    /// Get package info
    pub fn get_package(&self, platform: &Platform) -> Option<&PackageInfo> {
        self.packages.get(platform)
    }

    /// Verify package integrity
    pub fn verify_package(&self, package_path: &Path) -> RobinResult<bool> {
        // Would verify checksums, signatures, etc.
        Ok(package_path.exists())
    }
}

/// Package information
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub platform: Platform,
    pub file_path: PathBuf,
    pub size: u64,
    pub checksum: String,
    pub signature: Option<String>,
    pub created_at: SystemTime,
}

use std::time::SystemTime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_config() {
        let config = InstallerConfig::default();
        assert_eq!(config.app_name, "Robin");
        assert_eq!(config.version, "1.0.0");
    }

    #[test]
    fn test_nsis_script_generation() {
        let config = InstallerConfig::default();
        let generator = InstallerGenerator::new(config);
        let script = generator.generate_nsis_script().unwrap();
        assert!(script.contains("Robin"));
        assert!(script.contains("InstallDir"));
    }

    #[test]
    fn test_info_plist_generation() {
        let config = InstallerConfig::default();
        let generator = InstallerGenerator::new(config);
        let plist = generator.generate_info_plist().unwrap();
        assert!(plist.contains("CFBundleExecutable"));
        assert!(plist.contains("Robin"));
    }

    #[test]
    fn test_desktop_file_generation() {
        let config = InstallerConfig::default();
        let generator = InstallerGenerator::new(config);
        let desktop = generator.generate_desktop_file().unwrap();
        assert!(desktop.contains("[Desktop Entry]"));
        assert!(desktop.contains("Name=Robin"));
    }
}