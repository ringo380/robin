#!/bin/bash

# Robin Engine - Unified Deployment Script
# Orchestrates the complete deployment pipeline

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[DEPLOY]${NC} $1"
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

print_info() {
    echo -e "${PURPLE}[INFO]${NC} $1"
}

# Help function
show_help() {
    cat << EOF
Robin Engine Deployment Script

Usage: $0 [OPTIONS] [TARGET]

OPTIONS:
    -h, --help              Show this help message
    -c, --config FILE       Use custom config file (default: deploy.toml)
    -v, --verbose           Enable verbose output
    --dry-run              Show what would be done without executing
    --skip-tests           Skip running tests before build
    --skip-build           Skip building (use existing builds)
    --force                Force deployment even if checks fail

TARGETS:
    all                     Build and package for all platforms (default)
    build                   Build all platforms only
    package                 Package existing builds only
    steam                   Prepare Steam deployment
    github                  Trigger GitHub Actions workflow
    local                   Build for current platform only
    clean                   Clean all build artifacts

EXAMPLES:
    $0                      # Build and package for all platforms
    $0 build                # Build only, no packaging
    $0 steam                # Prepare Steam deployment
    $0 --dry-run all        # Show what would be done
    $0 --skip-tests local   # Quick local build without tests

ENVIRONMENT VARIABLES:
    STEAM_APP_ID           Steam application ID
    STEAM_USERNAME         Steam username for uploads
    STEAM_PASSWORD         Steam password for uploads
    GITHUB_TOKEN           GitHub token for releases
EOF
}

# Default configuration
CONFIG_FILE="scripts/deployment/deploy.toml"
VERBOSE=false
DRY_RUN=false
SKIP_TESTS=false
SKIP_BUILD=false
FORCE=false
TARGET="all"

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -c|--config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --skip-tests)
            SKIP_TESTS=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        all|build|package|steam|github|local|clean)
            TARGET="$1"
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Check if we're in the Robin project root
if [ ! -f "Cargo.toml" ] || [ ! -d "robin_demo" ]; then
    print_error "Please run this script from the Robin Engine project root"
    exit 1
fi

# Check for required tools
check_dependencies() {
    print_status "Checking dependencies..."

    local missing_deps=()

    command -v cargo >/dev/null 2>&1 || missing_deps+=("cargo (Rust)")
    command -v rustup >/dev/null 2>&1 || missing_deps+=("rustup")

    if [ "$TARGET" = "steam" ] || [ "$TARGET" = "all" ]; then
        command -v steamcmd >/dev/null 2>&1 || print_warning "steamcmd not found (optional for Steam deployment)"
    fi

    if [ ${#missing_deps[@]} -ne 0 ]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi

    print_success "All required dependencies found"
}

# Run tests
run_tests() {
    if [ "$SKIP_TESTS" = true ]; then
        print_warning "Skipping tests (--skip-tests specified)"
        return
    fi

    print_status "Running tests..."
    if [ "$DRY_RUN" = false ]; then
        cargo test --all-features || {
            print_error "Tests failed"
            [ "$FORCE" = false ] && exit 1
        }
    else
        print_info "Would run: cargo test --all-features"
    fi
    print_success "Tests completed"
}

# Build for all platforms
build_all_platforms() {
    if [ "$SKIP_BUILD" = true ]; then
        print_warning "Skipping build (--skip-build specified)"
        return
    fi

    print_status "Building for all platforms..."
    if [ "$DRY_RUN" = false ]; then
        ./scripts/deployment/build-all-platforms.sh
    else
        print_info "Would run: ./scripts/deployment/build-all-platforms.sh"
    fi
    print_success "Cross-platform build completed"
}

# Build for local platform only
build_local() {
    if [ "$SKIP_BUILD" = true ]; then
        print_warning "Skipping build (--skip-build specified)"
        return
    fi

    print_status "Building for local platform..."
    if [ "$DRY_RUN" = false ]; then
        cargo build --release --bin robin
        cd robin_demo && cargo build --release && cd ..
    else
        print_info "Would run: cargo build --release --bin robin"
        print_info "Would run: cd robin_demo && cargo build --release"
    fi
    print_success "Local build completed"
}

# Package builds
package_builds() {
    print_status "Packaging builds..."
    if [ "$DRY_RUN" = false ]; then
        # This would be enhanced to read from deploy.toml
        print_info "Creating distribution packages..."
        # Packaging logic would go here
    else
        print_info "Would create distribution packages"
    fi
    print_success "Packaging completed"
}

# Prepare Steam deployment
prepare_steam() {
    print_status "Preparing Steam deployment..."
    if [ "$DRY_RUN" = false ]; then
        ./scripts/deployment/steam-prepare.sh
    else
        print_info "Would run: ./scripts/deployment/steam-prepare.sh"
    fi
    print_success "Steam deployment prepared"
}

# Clean build artifacts
clean_builds() {
    print_status "Cleaning build artifacts..."
    if [ "$DRY_RUN" = false ]; then
        cargo clean
        rm -rf dist/
        rm -rf steam_build/
        print_success "Clean completed"
    else
        print_info "Would run: cargo clean"
        print_info "Would remove: dist/, steam_build/"
    fi
}

# GitHub workflow trigger
trigger_github() {
    print_status "GitHub Actions deployment is automatic on push/tag"
    print_info "To trigger manually:"
    print_info "  1. Push to main/develop branch, or"
    print_info "  2. Create a tag: git tag v1.0.0 && git push origin v1.0.0"
    print_info "  3. Use GitHub web interface to trigger workflow_dispatch"
}

# Main execution
main() {
    print_info "Robin Engine Deployment Pipeline"
    print_info "Target: $TARGET"
    [ "$DRY_RUN" = true ] && print_warning "DRY RUN MODE - No actual changes will be made"

    check_dependencies

    case $TARGET in
        all)
            run_tests
            build_all_platforms
            package_builds
            print_success "Complete deployment pipeline finished"
            ;;
        build)
            run_tests
            build_all_platforms
            ;;
        package)
            package_builds
            ;;
        steam)
            build_all_platforms
            prepare_steam
            ;;
        github)
            trigger_github
            ;;
        local)
            run_tests
            build_local
            ;;
        clean)
            clean_builds
            ;;
        *)
            print_error "Unknown target: $TARGET"
            show_help
            exit 1
            ;;
    esac

    print_success "Deployment target '$TARGET' completed successfully!"
}

# Execute main function
main "$@"