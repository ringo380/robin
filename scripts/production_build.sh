#!/bin/bash

# Production Build Script for Robin Engine
# Comprehensive build automation with optimization and packaging

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BUILD_OUTPUT_DIR="${PROJECT_ROOT}/target/production"
VERSION="${ROBIN_VERSION:-1.0.0}"
BUILD_TYPE="${BUILD_TYPE:-release}"
TARGET_PLATFORM="${TARGET_PLATFORM:-macos-universal}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Print banner
print_banner() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                    Robin Engine Production Build             ║"
    echo "║                         Version: $VERSION                        ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust toolchain."
        exit 1
    fi

    # Check required tools based on platform
    case "$TARGET_PLATFORM" in
        macos-*)
            if ! command -v codesign &> /dev/null; then
                log_warning "codesign not found. Code signing will be skipped."
            fi
            ;;
        windows-*)
            if ! command -v signtool &> /dev/null; then
                log_warning "signtool not found. Code signing will be skipped."
            fi
            ;;
    esac

    # Check disk space (require at least 2GB)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        available_space=$(df -g . | tail -1 | awk '{print $4}')
        if [ "$available_space" -lt 2 ]; then
            log_error "Insufficient disk space. At least 2GB required."
            exit 1
        fi
    fi

    log_success "Prerequisites check completed"
}

# Clean build environment
clean_build() {
    log_info "Cleaning build environment..."

    # Clean Cargo target directory
    cargo clean

    # Clean production output directory
    rm -rf "$BUILD_OUTPUT_DIR"
    mkdir -p "$BUILD_OUTPUT_DIR"

    log_success "Build environment cleaned"
}

# Build Robin Engine
build_engine() {
    log_info "Building Robin Engine..."

    cd "$PROJECT_ROOT"

    # Set optimization flags
    export RUSTFLAGS="-C target-cpu=native -C opt-level=3 -C lto=fat -C codegen-units=1"

    # Build based on platform
    case "$TARGET_PLATFORM" in
        macos-universal)
            log_info "Building universal macOS binary..."
            cargo build --release --target x86_64-apple-darwin
            cargo build --release --target aarch64-apple-darwin

            # Create universal binary
            lipo -create \
                target/x86_64-apple-darwin/release/robin \
                target/aarch64-apple-darwin/release/robin \
                -output "$BUILD_OUTPUT_DIR/robin"
            ;;
        macos-x64)
            log_info "Building x64 macOS binary..."
            cargo build --release --target x86_64-apple-darwin
            cp target/x86_64-apple-darwin/release/robin "$BUILD_OUTPUT_DIR/robin"
            ;;
        macos-arm64)
            log_info "Building ARM64 macOS binary..."
            cargo build --release --target aarch64-apple-darwin
            cp target/aarch64-apple-darwin/release/robin "$BUILD_OUTPUT_DIR/robin"
            ;;
        windows-x64)
            log_info "Building x64 Windows binary..."
            cargo build --release --target x86_64-pc-windows-msvc
            cp target/x86_64-pc-windows-msvc/release/robin.exe "$BUILD_OUTPUT_DIR/robin.exe"
            ;;
        linux-x64)
            log_info "Building x64 Linux binary..."
            cargo build --release --target x86_64-unknown-linux-gnu
            cp target/x86_64-unknown-linux-gnu/release/robin "$BUILD_OUTPUT_DIR/robin"
            ;;
        wasm)
            log_info "Building WebAssembly..."
            cargo build --release --target wasm32-unknown-unknown
            cp target/wasm32-unknown-unknown/release/robin.wasm "$BUILD_OUTPUT_DIR/robin.wasm"
            ;;
        *)
            log_error "Unknown target platform: $TARGET_PLATFORM"
            exit 1
            ;;
    esac

    log_success "Engine build completed"
}

# Optimize binary
optimize_binary() {
    log_info "Optimizing binary..."

    case "$TARGET_PLATFORM" in
        macos-*)
            # Strip debug symbols if not needed
            if [ "$BUILD_TYPE" = "release" ]; then
                strip "$BUILD_OUTPUT_DIR/robin"
            fi
            ;;
        windows-*)
            # Windows optimization would go here
            ;;
        linux-*)
            # Strip debug symbols if not needed
            if [ "$BUILD_TYPE" = "release" ]; then
                strip "$BUILD_OUTPUT_DIR/robin"
            fi
            ;;
    esac

    log_success "Binary optimization completed"
}

# Process assets
process_assets() {
    log_info "Processing assets..."

    local assets_dir="$PROJECT_ROOT/assets"
    local output_assets_dir="$BUILD_OUTPUT_DIR/assets"

    if [ -d "$assets_dir" ]; then
        mkdir -p "$output_assets_dir"

        # Copy and compress assets
        find "$assets_dir" -type f \( -name "*.png" -o -name "*.jpg" -o -name "*.wav" -o -name "*.ogg" \) | while read -r file; do
            relative_path="${file#$assets_dir/}"
            output_file="$output_assets_dir/$relative_path"
            output_dir="$(dirname "$output_file")"

            mkdir -p "$output_dir"

            # Copy asset (in production, this would include compression)
            cp "$file" "$output_file"
        done

        log_success "Asset processing completed"
    else
        log_warning "No assets directory found"
    fi
}

# Code signing
sign_binary() {
    if [ -z "${SIGNING_IDENTITY:-}" ]; then
        log_warning "No signing identity provided. Skipping code signing."
        return 0
    fi

    log_info "Signing binary..."

    case "$TARGET_PLATFORM" in
        macos-*)
            if command -v codesign &> /dev/null; then
                codesign --force --sign "$SIGNING_IDENTITY" "$BUILD_OUTPUT_DIR/robin"
                log_success "Code signing completed"
            else
                log_warning "codesign not available. Skipping code signing."
            fi
            ;;
        windows-*)
            if command -v signtool &> /dev/null; then
                signtool sign /f "$SIGNING_IDENTITY" "$BUILD_OUTPUT_DIR/robin.exe"
                log_success "Code signing completed"
            else
                log_warning "signtool not available. Skipping code signing."
            fi
            ;;
        *)
            log_info "Code signing not applicable for platform: $TARGET_PLATFORM"
            ;;
    esac
}

# Create distribution package
create_package() {
    log_info "Creating distribution package..."

    local package_name="robin-engine-${VERSION}-${TARGET_PLATFORM}"
    local package_dir="$BUILD_OUTPUT_DIR/$package_name"

    mkdir -p "$package_dir"

    # Copy binary
    case "$TARGET_PLATFORM" in
        macos-*)
            cp "$BUILD_OUTPUT_DIR/robin" "$package_dir/"

            # Create macOS app bundle if needed
            if [ "${CREATE_APP_BUNDLE:-false}" = "true" ]; then
                create_macos_app_bundle "$package_dir"
            fi
            ;;
        windows-*)
            cp "$BUILD_OUTPUT_DIR/robin.exe" "$package_dir/"
            ;;
        linux-*)
            cp "$BUILD_OUTPUT_DIR/robin" "$package_dir/"
            ;;
        wasm)
            cp "$BUILD_OUTPUT_DIR/robin.wasm" "$package_dir/"
            ;;
    esac

    # Copy assets
    if [ -d "$BUILD_OUTPUT_DIR/assets" ]; then
        cp -r "$BUILD_OUTPUT_DIR/assets" "$package_dir/"
    fi

    # Copy documentation
    if [ -f "$PROJECT_ROOT/README.md" ]; then
        cp "$PROJECT_ROOT/README.md" "$package_dir/"
    fi

    # Copy license
    if [ -f "$PROJECT_ROOT/LICENSE" ]; then
        cp "$PROJECT_ROOT/LICENSE" "$package_dir/"
    fi

    # Create archive
    cd "$BUILD_OUTPUT_DIR"
    case "$TARGET_PLATFORM" in
        macos-*)
            tar -czf "${package_name}.tar.gz" "$package_name"
            ;;
        windows-*)
            zip -r "${package_name}.zip" "$package_name"
            ;;
        linux-*)
            tar -czf "${package_name}.tar.gz" "$package_name"
            ;;
        wasm)
            tar -czf "${package_name}.tar.gz" "$package_name"
            ;;
    esac

    log_success "Distribution package created: ${package_name}"
}

# Create macOS app bundle
create_macos_app_bundle() {
    local package_dir="$1"
    local app_name="Robin Engine.app"
    local app_dir="$package_dir/$app_name"

    mkdir -p "$app_dir/Contents/MacOS"
    mkdir -p "$app_dir/Contents/Resources"

    # Move binary to app bundle
    mv "$package_dir/robin" "$app_dir/Contents/MacOS/robin"

    # Create Info.plist
    cat > "$app_dir/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>robin</string>
    <key>CFBundleIdentifier</key>
    <string>com.robinengine.app</string>
    <key>CFBundleName</key>
    <string>Robin Engine</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
EOF

    # Move assets to Resources
    if [ -d "$package_dir/assets" ]; then
        mv "$package_dir/assets" "$app_dir/Contents/Resources/"
    fi

    log_success "macOS app bundle created"
}

# Run tests
run_tests() {
    if [ "${SKIP_TESTS:-false}" = "true" ]; then
        log_warning "Skipping tests (SKIP_TESTS=true)"
        return 0
    fi

    log_info "Running tests..."

    cd "$PROJECT_ROOT"
    cargo test --release

    log_success "Tests completed"
}

# Verify build
verify_build() {
    log_info "Verifying build..."

    # Check that binary exists and is executable
    case "$TARGET_PLATFORM" in
        macos-*|linux-*)
            if [ ! -x "$BUILD_OUTPUT_DIR/robin" ]; then
                log_error "Binary is not executable or doesn't exist"
                exit 1
            fi
            ;;
        windows-*)
            if [ ! -f "$BUILD_OUTPUT_DIR/robin.exe" ]; then
                log_error "Binary doesn't exist"
                exit 1
            fi
            ;;
        wasm)
            if [ ! -f "$BUILD_OUTPUT_DIR/robin.wasm" ]; then
                log_error "WebAssembly module doesn't exist"
                exit 1
            fi
            ;;
    esac

    # Check binary size (warn if too large)
    if [[ "$TARGET_PLATFORM" != "wasm" ]]; then
        local binary_file="$BUILD_OUTPUT_DIR/robin"
        [ "$TARGET_PLATFORM" = "windows-x64" ] && binary_file="$BUILD_OUTPUT_DIR/robin.exe"

        local size_mb=$(du -m "$binary_file" | cut -f1)
        if [ "$size_mb" -gt 100 ]; then
            log_warning "Binary size is large: ${size_mb}MB"
        else
            log_info "Binary size: ${size_mb}MB"
        fi
    fi

    log_success "Build verification completed"
}

# Print build summary
print_summary() {
    local build_end_time=$(date)
    local build_duration=$(($(date +%s) - start_time))

    echo -e "\n${GREEN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                     BUILD SUMMARY                            ║${NC}"
    echo -e "${GREEN}╠══════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${GREEN}║${NC} Version:        $VERSION"
    echo -e "${GREEN}║${NC} Platform:       $TARGET_PLATFORM"
    echo -e "${GREEN}║${NC} Build Type:     $BUILD_TYPE"
    echo -e "${GREEN}║${NC} Duration:       ${build_duration}s"
    echo -e "${GREEN}║${NC} Output Dir:     $BUILD_OUTPUT_DIR"
    echo -e "${GREEN}║${NC} End Time:       $build_end_time"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════╝${NC}"

    # List output files
    echo -e "\n${BLUE}Output Files:${NC}"
    find "$BUILD_OUTPUT_DIR" -type f -name "*.tar.gz" -o -name "*.zip" | while read -r file; do
        echo "  📦 $(basename "$file")"
    done
}

# Main execution
main() {
    local start_time=$(date +%s)

    print_banner

    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --platform)
                TARGET_PLATFORM="$2"
                shift 2
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --skip-tests)
                SKIP_TESTS=true
                shift
                ;;
            --app-bundle)
                CREATE_APP_BUNDLE=true
                shift
                ;;
            --signing-identity)
                SIGNING_IDENTITY="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo "Options:"
                echo "  --platform PLATFORM    Target platform (macos-universal, macos-x64, macos-arm64, windows-x64, linux-x64, wasm)"
                echo "  --version VERSION       Build version (default: 1.0.0)"
                echo "  --skip-tests           Skip running tests"
                echo "  --app-bundle           Create macOS app bundle"
                echo "  --signing-identity ID   Code signing identity"
                echo "  --help                 Show this help message"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    # Execute build pipeline
    check_prerequisites
    clean_build
    run_tests
    build_engine
    optimize_binary
    process_assets
    sign_binary
    create_package
    verify_build
    print_summary

    log_success "Production build completed successfully! 🎉"
}

# Execute main function with all arguments
main "$@"