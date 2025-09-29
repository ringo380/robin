#!/bin/bash

# Robin Engine - Steam Deployment Preparation Script
# Prepares builds for Steam distribution using SteamCMD

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[STEAM]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[STEAM SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[STEAM WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[STEAM ERROR]${NC} $1"
}

# Configuration
STEAM_APP_ID="${STEAM_APP_ID:-YOUR_APP_ID}"
STEAM_USERNAME="${STEAM_USERNAME:-your_steam_username}"
STEAM_BUILD_DESC="${STEAM_BUILD_DESC:-Robin Engine Auto-Build $(date '+%Y-%m-%d %H:%M')}"

# Check if we're in the Robin project root
if [ ! -f "Cargo.toml" ] || [ ! -d "robin_demo" ]; then
    print_error "Please run this script from the Robin Engine project root"
    exit 1
fi

# Check for Steam credentials
if [ "$STEAM_APP_ID" = "YOUR_APP_ID" ]; then
    print_error "Please set STEAM_APP_ID environment variable"
    print_status "Example: export STEAM_APP_ID=123456"
    exit 1
fi

print_status "Preparing Steam build for App ID: $STEAM_APP_ID"

# Create Steam build directory structure
print_status "Creating Steam build structure..."
mkdir -p steam_build/{content/{linux,windows,macos},scripts}

# Copy builds to Steam content directories
print_status "Copying platform builds..."

# Linux build
if [ -d "dist/linux" ] && [ -f "dist/linux/robin_demo" ]; then
    cp -r dist/linux/* steam_build/content/linux/
    print_success "Linux build copied"
else
    print_warning "Linux build not found in dist/linux/"
fi

# Windows build
if [ -d "dist/windows" ] && [ -f "dist/windows/robin_demo.exe" ]; then
    cp -r dist/windows/* steam_build/content/windows/
    print_success "Windows build copied"
else
    print_warning "Windows build not found in dist/windows/"
fi

# macOS build (use ARM64 as primary)
if [ -d "dist/macos-arm64" ] && [ -f "dist/macos-arm64/robin_demo" ]; then
    cp -r dist/macos-arm64/* steam_build/content/macos/
    print_success "macOS build copied"
elif [ -d "dist/macos-x86" ] && [ -f "dist/macos-x86/robin_demo" ]; then
    cp -r dist/macos-x86/* steam_build/content/macos/
    print_success "macOS x86 build copied (ARM64 preferred)"
else
    print_warning "macOS build not found"
fi

# Create Steam app build script
print_status "Creating Steam app build script..."
cat > steam_build/scripts/app_build.vdf << EOF
"appbuild"
{
    "appid" "$STEAM_APP_ID"
    "desc" "$STEAM_BUILD_DESC"
    "buildoutput" "../../steam_output"
    "contentroot" "../content"
    "setlive" "default"

    "depots"
    {
        // Linux depot
        "${STEAM_APP_ID}01"
        {
            "FileMapping"
            {
                "LocalPath" "linux/*"
                "DepotPath" "."
                "recursive" "1"
            }
            "FileExclusion" "*.pdb"
        }

        // Windows depot
        "${STEAM_APP_ID}02"
        {
            "FileMapping"
            {
                "LocalPath" "windows/*"
                "DepotPath" "."
                "recursive" "1"
            }
            "FileExclusion" "*.pdb"
        }

        // macOS depot
        "${STEAM_APP_ID}03"
        {
            "FileMapping"
            {
                "LocalPath" "macos/*"
                "DepotPath" "."
                "recursive" "1"
            }
            "FileExclusion" "*.dSYM"
        }
    }
}
EOF

# Create depot build scripts
print_status "Creating depot build scripts..."

# Linux depot
cat > steam_build/scripts/depot_build_${STEAM_APP_ID}01.vdf << EOF
"DepotBuildConfig"
{
    "DepotID" "${STEAM_APP_ID}01"
    "ContentRoot" "../content/linux"
    "FileMapping"
    {
        "LocalPath" "*"
        "DepotPath" "."
        "recursive" "1"
    }
    "FileExclusion" "*.debug"
}
EOF

# Windows depot
cat > steam_build/scripts/depot_build_${STEAM_APP_ID}02.vdf << EOF
"DepotBuildConfig"
{
    "DepotID" "${STEAM_APP_ID}02"
    "ContentRoot" "../content/windows"
    "FileMapping"
    {
        "LocalPath" "*"
        "DepotPath" "."
        "recursive" "1"
    }
    "FileExclusion" "*.pdb"
}
EOF

# macOS depot
cat > steam_build/scripts/depot_build_${STEAM_APP_ID}03.vdf << EOF
"DepotBuildConfig"
{
    "DepotID" "${STEAM_APP_ID}03"
    "ContentRoot" "../content/macos"
    "FileMapping"
    {
        "LocalPath" "*"
        "DepotPath" "."
        "recursive" "1"
    }
    "FileExclusion" "*.dSYM"
}
EOF

# Create upload script
print_status "Creating Steam upload script..."
cat > steam_build/upload_to_steam.sh << 'EOF'
#!/bin/bash

# Robin Engine Steam Upload Script
# Uploads the build to Steam using SteamCMD

set -e

# Check for SteamCMD
if ! command -v steamcmd >/dev/null 2>&1; then
    echo "Error: SteamCMD not found. Please install SteamCMD first."
    echo "Download from: https://developer.valvesoftware.com/wiki/SteamCMD"
    exit 1
fi

# Check for required environment variables
if [ -z "$STEAM_USERNAME" ]; then
    echo "Error: STEAM_USERNAME environment variable not set"
    exit 1
fi

if [ -z "$STEAM_PASSWORD" ]; then
    echo "Error: STEAM_PASSWORD environment variable not set"
    echo "Note: Use Steam Guard app codes or set up Steam Guard file"
    exit 1
fi

echo "Starting Steam upload..."
echo "App ID: $STEAM_APP_ID"
echo "Username: $STEAM_USERNAME"
echo "Build Description: $STEAM_BUILD_DESC"

# Create output directory
mkdir -p ../steam_output

# Upload to Steam
steamcmd +login "$STEAM_USERNAME" "$STEAM_PASSWORD" \
         +run_app_build_http scripts/app_build.vdf \
         +quit

echo "Steam upload completed!"
echo "Check the Steam Partner site for build status."
EOF

chmod +x steam_build/upload_to_steam.sh

# Create verification script
print_status "Creating build verification script..."
cat > steam_build/verify_build.sh << 'EOF'
#!/bin/bash

# Robin Engine Steam Build Verification
# Verifies the Steam build structure and content

set -e

echo "Verifying Steam build structure..."

# Check content directories
for platform in linux windows macos; do
    if [ -d "content/$platform" ]; then
        echo "✓ $platform content directory exists"

        # Check for main executable
        if [ "$platform" = "windows" ]; then
            if [ -f "content/$platform/robin_demo.exe" ]; then
                echo "✓ $platform executable found"
            else
                echo "✗ $platform executable missing"
            fi
        else
            if [ -f "content/$platform/robin_demo" ]; then
                echo "✓ $platform executable found"
            else
                echo "✗ $platform executable missing"
            fi
        fi

        # Check file sizes
        size=$(du -sh "content/$platform" | cut -f1)
        echo "  Build size: $size"
    else
        echo "✗ $platform content directory missing"
    fi
done

# Check scripts
if [ -f "scripts/app_build.vdf" ]; then
    echo "✓ App build script exists"
else
    echo "✗ App build script missing"
fi

# Validate VDF files
echo "Validating VDF files..."
for vdf in scripts/*.vdf; do
    if [ -f "$vdf" ]; then
        if grep -q "appid\|DepotID" "$vdf"; then
            echo "✓ $(basename "$vdf") appears valid"
        else
            echo "✗ $(basename "$vdf") may be invalid"
        fi
    fi
done

echo "Steam build verification completed!"
EOF

chmod +x steam_build/verify_build.sh

# Create Steam deployment instructions
print_status "Creating deployment instructions..."
cat > steam_build/STEAM_DEPLOYMENT.md << EOF
# Robin Engine Steam Deployment Guide

## Prerequisites

1. **SteamCMD Installation**
   - Download from: https://developer.valvesoftware.com/wiki/SteamCMD
   - Add to PATH or use full path

2. **Steam Partner Account**
   - App ID configured in Steamworks
   - Depot IDs created for each platform:
     - Linux: ${STEAM_APP_ID}01
     - Windows: ${STEAM_APP_ID}02
     - macOS: ${STEAM_APP_ID}03

3. **Environment Variables**
   \`\`\`bash
   export STEAM_APP_ID="$STEAM_APP_ID"
   export STEAM_USERNAME="your_steam_username"
   export STEAM_PASSWORD="your_steam_password"
   export STEAM_BUILD_DESC="Build description"
   \`\`\`

## Build Process

1. **Prepare builds** (run from project root):
   \`\`\`bash
   # Build all platforms
   ./scripts/deployment/build-all-platforms.sh

   # Prepare Steam build
   ./scripts/deployment/steam-prepare.sh
   \`\`\`

2. **Verify build**:
   \`\`\`bash
   cd steam_build
   ./verify_build.sh
   \`\`\`

3. **Upload to Steam**:
   \`\`\`bash
   cd steam_build
   ./upload_to_steam.sh
   \`\`\`

## File Structure

\`\`\`
steam_build/
├── content/
│   ├── linux/          # Linux build files
│   ├── windows/        # Windows build files
│   └── macos/          # macOS build files
├── scripts/
│   ├── app_build.vdf   # Main build configuration
│   └── depot_build_*.vdf # Individual depot configs
├── upload_to_steam.sh  # Upload script
├── verify_build.sh     # Verification script
└── STEAM_DEPLOYMENT.md # This file
\`\`\`

## Steam Guard

For automated builds, you'll need to:
1. Use Steam Guard mobile authenticator, or
2. Set up Steam Guard file authentication
3. Consider using Steam Build Account for CI/CD

## Troubleshooting

- **Login Issues**: Check Steam Guard setup
- **Build Fails**: Verify depot IDs in Steamworks
- **File Missing**: Run verify_build.sh first
- **Upload Timeout**: Check network and Steam status

## Automation

For CI/CD integration, add these secrets:
- \`STEAM_USERNAME\`
- \`STEAM_PASSWORD\`
- \`STEAM_APP_ID\`
- \`STEAM_BUILD_DESC\` (optional)
EOF

print_success "Steam deployment preparation completed!"
print_status "Steam build structure created in steam_build/"
print_status "Next steps:"
echo "  1. Review steam_build/STEAM_DEPLOYMENT.md"
echo "  2. Configure your Steam app settings"
echo "  3. Set environment variables"
echo "  4. Run verification: cd steam_build && ./verify_build.sh"
echo "  5. Upload to Steam: cd steam_build && ./upload_to_steam.sh"

print_warning "Remember to:"
echo "  - Replace $STEAM_APP_ID with your actual Steam App ID"
echo "  - Configure depot IDs in Steamworks"
echo "  - Set up Steam Guard for automated uploads"