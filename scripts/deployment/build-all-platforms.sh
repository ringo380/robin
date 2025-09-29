#!/bin/bash

# Robin Engine - Cross-Platform Build Script
# Builds Robin Engine for all supported platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the Robin project root
if [ ! -f "Cargo.toml" ] || [ ! -d "robin_demo" ]; then
    print_error "Please run this script from the Robin Engine project root"
    exit 1
fi

# Create dist directory
print_status "Creating distribution directory..."
mkdir -p dist/{linux,windows,macos-arm64,macos-x86}

# Build Linux (x86_64)
print_status "Building for Linux x86_64..."
if command -v cargo >/dev/null 2>&1; then
    # Add Linux target if not present
    rustup target add x86_64-unknown-linux-gnu 2>/dev/null || true

    # Build main engine
    cargo build --release --target x86_64-unknown-linux-gnu --bin robin

    # Build demo
    cd robin_demo
    cargo build --release --target x86_64-unknown-linux-gnu
    cd ..

    # Package Linux build
    cp target/x86_64-unknown-linux-gnu/release/robin dist/linux/
    cp robin_demo/target/x86_64-unknown-linux-gnu/release/robin_demo dist/linux/
    cp README.md dist/linux/ 2>/dev/null || echo "# Robin Engine" > dist/linux/README.md

    print_success "Linux build completed"
else
    print_error "Cargo not found. Please install Rust."
    exit 1
fi

# Build Windows (x86_64) - cross compilation
print_status "Building for Windows x86_64..."
if rustup target list | grep -q "x86_64-pc-windows-gnu"; then
    rustup target add x86_64-pc-windows-gnu 2>/dev/null || true

    # Note: This requires mingw-w64 for cross compilation
    if command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        export CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
        export CXX_x86_64_pc_windows_gnu=x86_64-w64-mingw32-g++

        cargo build --release --target x86_64-pc-windows-gnu --bin robin

        cd robin_demo
        cargo build --release --target x86_64-pc-windows-gnu
        cd ..

        cp target/x86_64-pc-windows-gnu/release/robin.exe dist/windows/
        cp robin_demo/target/x86_64-pc-windows-gnu/release/robin_demo.exe dist/windows/
        cp README.md dist/windows/ 2>/dev/null || echo "# Robin Engine" > dist/windows/README.md

        print_success "Windows build completed"
    else
        print_warning "mingw-w64 not found. Skipping Windows cross-compilation."
        print_warning "To build for Windows, install: sudo apt-get install gcc-mingw-w64"
    fi
else
    print_warning "Windows target not available. Skipping Windows build."
fi

# Build macOS (if on macOS)
if [[ "$OSTYPE" == "darwin"* ]]; then
    print_status "Building for macOS (ARM64 and x86_64)..."

    # ARM64 build
    rustup target add aarch64-apple-darwin 2>/dev/null || true
    cargo build --release --target aarch64-apple-darwin --bin robin

    cd robin_demo
    cargo build --release --target aarch64-apple-darwin
    cd ..

    cp target/aarch64-apple-darwin/release/robin dist/macos-arm64/
    cp robin_demo/target/aarch64-apple-darwin/release/robin_demo dist/macos-arm64/
    cp README.md dist/macos-arm64/ 2>/dev/null || echo "# Robin Engine" > dist/macos-arm64/README.md

    # x86_64 build
    rustup target add x86_64-apple-darwin 2>/dev/null || true
    cargo build --release --target x86_64-apple-darwin --bin robin

    cd robin_demo
    cargo build --release --target x86_64-apple-darwin
    cd ..

    cp target/x86_64-apple-darwin/release/robin dist/macos-x86/
    cp robin_demo/target/x86_64-apple-darwin/release/robin_demo dist/macos-x86/
    cp README.md dist/macos-x86/ 2>/dev/null || echo "# Robin Engine" > dist/macos-x86/README.md

    print_success "macOS builds completed"
else
    print_warning "Not on macOS. Skipping macOS builds."
fi

# Create packages
print_status "Creating distribution packages..."

# Linux package
cd dist
if [ -d "linux" ] && [ -f "linux/robin" ]; then
    tar -czf robin-engine-linux-x86_64.tar.gz linux/
    print_success "Created robin-engine-linux-x86_64.tar.gz"
fi

# Windows package
if [ -d "windows" ] && [ -f "windows/robin.exe" ]; then
    zip -r robin-engine-windows-x86_64.zip windows/
    print_success "Created robin-engine-windows-x86_64.zip"
fi

# macOS packages
if [ -d "macos-arm64" ] && [ -f "macos-arm64/robin" ]; then
    tar -czf robin-engine-macos-arm64.tar.gz macos-arm64/
    print_success "Created robin-engine-macos-arm64.tar.gz"
fi

if [ -d "macos-x86" ] && [ -f "macos-x86/robin" ]; then
    tar -czf robin-engine-macos-x86_64.tar.gz macos-x86/
    print_success "Created robin-engine-macos-x86_64.tar.gz"
fi

cd ..

print_success "Build completed! Distribution packages are in the dist/ directory."
print_status "Available packages:"
ls -la dist/*.{tar.gz,zip} 2>/dev/null || print_warning "No packages created"

print_status "Build summary:"
echo "  - Linux x86_64: $([ -f dist/linux/robin ] && echo "✓" || echo "✗")"
echo "  - Windows x86_64: $([ -f dist/windows/robin.exe ] && echo "✓" || echo "✗")"
echo "  - macOS ARM64: $([ -f dist/macos-arm64/robin ] && echo "✓" || echo "✗")"
echo "  - macOS x86_64: $([ -f dist/macos-x86/robin ] && echo "✓" || echo "✗")"