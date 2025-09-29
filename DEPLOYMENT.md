# Robin Engine Deployment Guide

Complete deployment pipeline for distributing Robin Engine across multiple platforms and distribution channels.

## 🚀 Quick Start

```bash
# Complete deployment pipeline
./scripts/deployment/deploy.sh all

# Local development build
./scripts/deployment/deploy.sh local

# Steam deployment preparation
./scripts/deployment/deploy.sh steam

# Show what would happen without executing
./scripts/deployment/deploy.sh --dry-run all
```

## 📁 Deployment Pipeline Overview

The Robin Engine deployment system provides automated builds and distribution for:

- **Cross-Platform Builds**: Linux, Windows, macOS (ARM64 & x86_64)
- **Distribution Channels**: Steam, GitHub Releases, Direct Download
- **CI/CD Integration**: GitHub Actions with automated testing and builds
- **Quality Assurance**: Automated testing, formatting checks, and verification

## 🛠️ Tools and Scripts

### Main Deployment Script
- **`scripts/deployment/deploy.sh`** - Unified deployment orchestration
- **`scripts/deployment/deploy.toml`** - Centralized configuration

### Platform Building
- **`scripts/deployment/build-all-platforms.sh`** - Cross-platform build automation
- **`.cargo/config.toml`** - Rust build optimization for Apple Silicon

### Steam Integration
- **`scripts/deployment/steam-prepare.sh`** - Steam deployment preparation
- **Steam VDF files** - Automated generation for SteamCMD

### CI/CD Pipeline
- **`.github/workflows/ci-cd.yml`** - Complete GitHub Actions workflow

## 🏗️ Build Targets

### macOS (Native Platform)
- **ARM64**: `aarch64-apple-darwin` - Optimized for Apple Silicon
- **x86_64**: `x86_64-apple-darwin` - Intel Mac compatibility
- **Features**: Native Metal rendering, app bundle creation

### Linux
- **x86_64**: `x86_64-unknown-linux-gnu`
- **Features**: Wayland/X11 support, OpenGL rendering

### Windows
- **x86_64**: `x86_64-pc-windows-gnu` (cross-compilation)
- **Features**: DirectX/OpenGL rendering, executable packaging

## 📦 Distribution Formats

### Archives
- **Linux**: `robin-engine-linux-x86_64.tar.gz`
- **Windows**: `robin-engine-windows-x86_64.zip`
- **macOS ARM64**: `robin-engine-macos-arm64.tar.gz`
- **macOS x86**: `robin-engine-macos-x86_64.tar.gz`

### Steam
- **Content Structure**: Platform-specific depot organization
- **Automation**: SteamCMD integration with VDF generation
- **Verification**: Build validation and content verification

### GitHub Releases
- **Automated**: Triggered on version tags (`v*`)
- **Artifacts**: All platform builds attached to releases
- **Release Notes**: Auto-generated from commit history

## ⚙️ Configuration

### Environment Variables

#### Steam Deployment
```bash
export STEAM_APP_ID="your_steam_app_id"
export STEAM_USERNAME="your_steam_username"
export STEAM_PASSWORD="your_steam_password"
export STEAM_BUILD_DESC="Build description"
```

#### GitHub Integration
```bash
export GITHUB_TOKEN="your_github_token"  # For release creation
```

### Build Configuration
Edit `scripts/deployment/deploy.toml` for:
- Platform enable/disable flags
- Build optimization settings
- Steam depot configuration
- Packaging options

## 🔄 CI/CD Workflow

### Automated Triggers
1. **Pull Requests** → Run tests and build verification
2. **Push to main/develop** → Full platform builds
3. **Version tags** → Release creation and Steam deployment

### Workflow Steps
1. **Check Phase**: Formatting, linting, testing
2. **Build Phase**: Cross-platform compilation
3. **Package Phase**: Archive creation and validation
4. **Deploy Phase**: Steam upload and GitHub release

### Manual Triggers
```bash
# Local testing of CI pipeline
git tag v1.0.0-test
git push origin v1.0.0-test

# Production release
git tag v1.0.0
git push origin v1.0.0
```

## 🎯 Steam Deployment

### Prerequisites
1. **Steamworks Partner Account**
2. **App ID Configuration**
3. **Depot Setup**: Linux (01), Windows (02), macOS (03)
4. **SteamCMD Installation**

### Deployment Process
```bash
# 1. Build all platforms
./scripts/deployment/deploy.sh build

# 2. Prepare Steam build
./scripts/deployment/deploy.sh steam

# 3. Verify build structure
cd steam_build && ./verify_build.sh

# 4. Upload to Steam
cd steam_build && ./upload_to_steam.sh
```

### Steam Build Structure
```
steam_build/
├── content/
│   ├── linux/          # Linux build files
│   ├── windows/        # Windows build files
│   └── macos/          # macOS build files
├── scripts/
│   ├── app_build.vdf   # Main build config
│   └── depot_build_*.vdf # Platform depots
└── upload_to_steam.sh  # Upload automation
```

## 🧪 Quality Assurance

### Pre-Build Checks
- **Code Formatting**: `cargo fmt --check`
- **Linting**: `cargo clippy --all-targets`
- **Testing**: `cargo test --all-features`

### Build Verification
- **Binary Creation**: Executable generation validation
- **Size Limits**: Build size monitoring
- **Dependency Verification**: Library linking validation

### Steam Verification
- **Content Validation**: File structure and executable checks
- **VDF Syntax**: Steam configuration file validation
- **Depot Mapping**: Platform content organization

## 🚨 Troubleshooting

### Common Issues

#### Metal/Rosetta Compatibility
- **Issue**: Foreign exception crashes on Apple Silicon
- **Solution**: Use native ARM64 builds (`aarch64-apple-darwin`)
- **Config**: Automatic via `.cargo/config.toml`

#### Cross-Compilation Failures
- **Windows**: Requires `mingw-w64` for cross-compilation
- **Install**: `sudo apt-get install gcc-mingw-w64`

#### Steam Upload Failures
- **Steam Guard**: Set up mobile authenticator or guard file
- **Network**: Check Steam service status
- **Credentials**: Verify environment variables

### Debug Commands
```bash
# Verbose deployment
./scripts/deployment/deploy.sh -v all

# Dry run to see what would happen
./scripts/deployment/deploy.sh --dry-run steam

# Skip tests for quick builds
./scripts/deployment/deploy.sh --skip-tests local

# Force deployment despite errors
./scripts/deployment/deploy.sh --force all
```

## 📊 Performance Metrics

### Build Optimization
- **LTO**: Link-time optimization enabled
- **Strip Symbols**: Release binaries stripped
- **Target CPU**: Platform-specific optimizations

### Apple Silicon Specific
- **Native Metal**: Hardware-accelerated rendering
- **Unified Memory**: Optimized memory management
- **M-series CPUs**: Target-specific instruction sets

## 🔮 Future Enhancements

### Planned Features
- **Itch.io Integration**: Automated itch.io uploads
- **Windows Store**: UWP packaging and submission
- **App Store**: macOS App Store preparation
- **Code Signing**: Automated certificate management
- **Crash Reporting**: Integrated crash collection
- **Analytics**: Usage metrics and telemetry

### Security Enhancements
- **Code Signing**: Windows and macOS executables
- **Notarization**: Apple notarization automation
- **Vulnerability Scanning**: Automated security audits
- **Dependency Auditing**: Supply chain security

## 📚 Resources

### Documentation
- [SteamCMD Documentation](https://developer.valvesoftware.com/wiki/SteamCMD)
- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)

### Tools
- [SteamCMD Download](https://developer.valvesoftware.com/wiki/SteamCMD)
- [Rustup Installation](https://rustup.rs/)
- [GitHub CLI](https://cli.github.com/)

---

**🎮 Ready to deploy Robin Engine across all platforms!**

For questions or issues, check the troubleshooting section above or create an issue in the repository.