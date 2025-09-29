# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Robin is a 3D voxel game engine** - built from scratch in Rust for innovative building and crafting gameplay. The engine features real-time 3D rendering, voxel terrain generation, and an Engineer Build Mode system for creative world construction. This is a **game engine first** - designed for players who enjoy construction, logic, and crafting mechanics.

**IMPORTANT**: Robin is NOT an educational platform. Any educational value emerges naturally from engaging gameplay mechanics, not from deliberate educational design. Focus on the game engine capabilities and innovative voxel construction systems.

## Critical User Requirements

**IMPORTANT**: The user explicitly requires real 3D graphics windows with interactive navigation - NOT ASCII-based terminal demos:
- Real first-person perspective with WASD movement controls
- Fully textured environments with dynamic lighting
- Interactive terrain following and realistic movement physics
- Visible sun/moon in skybox with realistic day/night cycles
- Real wgpu-based graphics windows that appear on macOS
- Voxel world construction with Engineer Build Mode capabilities

## Development Commands

### Building and Running
```bash
# Check for compilation errors without building
cargo check

# Build the project
cargo build

# Build in release mode for performance
cargo build --release

# Run linting
cargo clippy

# Run tests
cargo test

# Run specific test
cargo test test_name

# Run benchmarks
cargo bench

# Run with debug logging
RUST_LOG=debug cargo run

# Run with backtrace for debugging
RUST_BACKTRACE=full cargo run
```

### Current Development Workflow (Post-Consolidation)
```bash
# PRIMARY: Unified Robin Demo (RECOMMENDED)
cd robin_demo && cargo run              # Main demo with all current features

# SECONDARY: Testing sandbox for new features
cargo run --bin robin_test              # Lightweight prototyping environment

# TERTIARY: Main Robin Engine binary
cargo run --bin robin                   # Unified systems demo (architecture test)

# LEGACY: Historical 2D particle demos
cargo run magical                       # 2D magical effects demo
cargo run basic                         # Basic windowed demo
```

### Debugging Workflow
```bash
# Debug with full logging and backtraces
RUST_LOG=debug RUST_BACKTRACE=full cargo run --bin robin_test

# Timeout for hanging processes
timeout 30s cargo run --bin robin_test

# Quick compilation check
env CARGO_INCREMENTAL=0 RUSTFLAGS="-C opt-level=0" timeout 30s cargo check --lib
```

## Architecture Overview

### Post-Consolidation Structure

**The project successfully transitioned from 50+ scattered demos to a focused, unified architecture:**

1. **Core Engine** (`src/engine/`)
   - **Unified Systems**: All 35+ modules now integrated and operational
   - **Build Mode System**: Complete Engineer Build Mode with advanced construction tools
   - **AI Systems**: ML-powered content generation and NPC intelligence
   - **Performance Systems**: GPU acceleration, LOD, and chunk streaming
   - **3D Graphics**: Full wgpu rendering pipeline with PBR materials

2. **Demonstration Layer**
   - **`robin_demo/`**: Flagship unified demo showcasing all capabilities
   - **`robin_test.rs`**: Lightweight sandbox for feature prototyping
   - **`src/bin/robin.rs`**: Main engine binary for architecture testing
   - **`archive/demos/`**: 50+ historical demos preserved for reference

3. **Production Features** (Phases 1-3 Complete)
   - **3D Voxel Engine**: 92% frustum culling, 60-80% vertex reduction via greedy meshing
   - **Apple Silicon Optimization**: Native Metal rendering with unified memory support
   - **Engineer Build Mode**: Real-time world construction with advanced tools
   - **Interactive Particle Effects**: Physics-based particles for block placement/removal
   - **AI-Assisted Development**: Procedural generation and intelligent assistance
   - **Advanced Game Systems**: NPC intelligence, story systems, and dynamic world generation

### Key Dependencies (Updated)
- **wgpu 0.20**: WebGPU graphics API for all 3D rendering
- **winit 0.29**: Cross-platform windowing (critical for macOS)
- **rapier3d**: 3D physics engine for collision detection and particle physics
- **cgmath + nalgebra**: 3D math transformations and linear algebra
- **smartcore + ndarray**: Machine learning for AI systems
- **tokio**: Async runtime for networking and background processing
- **serde ecosystem**: Serialization for saves, configs, and networking
- **rusqlite + r2d2**: Database systems for asset management
- **rayon + crossbeam**: Parallel processing and lock-free data structures
- **metal** (macOS): Native Metal rendering optimization

## Code Organization

### Current Structure (Post-Consolidation)
```
src/
├── engine/                    # Core engine systems (35+ modules)
│   ├── build_mode/           # Engineer Build Mode system
│   ├── graphics/             # 3D rendering pipeline
│   ├── ai/                   # AI systems and ML integration
│   ├── world/                # Voxel world and construction
│   ├── performance/          # Optimization systems
│   ├── platform/             # Platform-specific code
│   └── [30+ other modules]   # Complete engine systems
├── bin/robin.rs              # Main engine binary
├── examples/                 # Example applications
└── main.rs                   # Legacy 2D demo entry point

robin_demo/                   # PRIMARY: Unified 3D demo
├── src/
│   ├── main.rs              # Main demo application
│   ├── renderer/            # Demo-specific rendering
│   └── ui/                  # Demo UI systems
└── Cargo.toml              # Independent build system

archive/demos/               # Historical demo preservation
├── phase2/                 # Phase 2 demos (8 files)
├── phase3/                 # Phase 3 demos (6 files)
├── tests/                  # Test demos (12 files)
└── legacy/                 # Legacy demos (18+ files)
```

### Critical Implementation Areas
- **Main Engine Binary**: `src/bin/robin.rs` - Complete engine with particle effects system
- **Unified Demo**: `robin_demo/src/main.rs` - Primary showcase of all capabilities
- **Engine Core**: `src/engine/mod.rs` - All 35+ modules now enabled and integrated
- **Build Mode**: `src/engine/build_mode/` - Complete Engineer Build Mode system
- **3D Graphics**: `src/engine/graphics/` + `robin_demo/src/renderer/` - Full pipeline
- **Physics & Particles**: `src/bin/robin.rs` - Rapier3d physics with particle effects
- **AI Systems**: `src/engine/ai/` - ML-powered content generation and assistance
- **Voxel Engine**: `src/engine/world/construction/voxel_engine.rs` - Core voxel systems
- **Platform Integration**: `src/engine/platform/` - macOS, Steam, mobile, web

## Current Development Status

**Phase 1**: Core Systems ✅ **COMPLETE**
- All foundational systems implemented and unified
- Working 3D graphics with native Metal rendering
- Character movement with terrain following physics
- Voxel-based world construction with Engineer Build Mode

**Phase 2**: Advanced Features ✅ **COMPLETE**
- Visual scripting and behavior trees integrated
- Multiplayer collaboration framework
- Performance optimization: 92% frustum culling, 60-80% vertex reduction
- Advanced graphics: PBR materials, dynamic lighting, particle systems
- Interactive particle effects: physics-based block construction feedback
- Audio and immersion systems with spatial audio

**Phase 3**: Polish and Distribution ✅ **COMPLETE**
- Demo consolidation: 50+ demos → unified structure
- Professional project organization and architecture
- Platform integration framework (macOS, Steam, mobile, web)
- Asset pipeline with database management

**Phase 4**: Production Ready 🔄 **CURRENT**
- Final UI polish and modern interface systems
- Advanced gameplay mechanics and player progression
- Multiplayer systems and collaborative building
- Platform deployment and distribution

## Demo Consolidation Achievement ✅

**Successfully resolved the scattered demo problem:**
- **Before**: 50+ standalone demo files creating maintenance burden
- **After**: Focused structure with `robin_demo/` as primary showcase
- **Preserved**: All historical demos archived in `archive/demos/`
- **Performance**: Maintained 92% frustum culling and greedy meshing optimizations
- **Development**: Clear workflow with `robin_test.rs` sandbox for prototyping

## Current Architecture Status

**All engine modules are now operational:**
- ✅ 35+ engine modules integrated and functioning
- ✅ Build Mode system with 10+ construction modes and 11+ templates
- ✅ AI systems with ML-powered content generation
- ✅ Performance systems with GPU acceleration and optimization
- ✅ Platform abstraction for multiple deployment targets
- ✅ Advanced game systems for immersive gameplay

## Graphics Requirements

When implementing 3D graphics:
- **ALWAYS** use wgpu for rendering, NOT ASCII/terminal graphics
- **ALWAYS** create real windows using winit that appear on macOS
- Implement proper lighting, materials, and textures
- Use first-person camera with WASD + mouse controls
- Support terrain-following physics with collision detection

## Voxel System Standards
- 32³ chunk size for optimal performance
- Face culling for hidden surfaces
- Multiple material types (Earth, Stone, Water, Grass, Sand)
- Proper mesh generation with vertex buffers
- Efficient spatial indexing for collision
- Physics-based particle effects on block placement/removal

## Interactive Particle Effects System

**Real-time particle feedback for voxel interactions:**
- **Block Placement**: 8 particles with upward velocity bias, color-matched to block material
- **Block Removal**: 12 particles with explosion pattern, simulating debris
- **Physics Integration**: Full Rapier3d physics simulation with gravity and collision
- **Visual Design**: Particle size, lifetime, and color vary by material type
- **Performance**: Efficient particle pooling with configurable maximum particle count

**Implementation Location:** `src/bin/robin.rs` - Integrated ParticleSystem with RobinApp

**Key Features:**
- Color-coded particles based on voxel material (Earth=brown, Stone=gray, Water=blue, etc.)
- Physics-realistic trajectories with gravity and initial velocity
- Lifetime management: particles fade over 2.0 seconds
- Wireframe rendering for optimal performance
- Integration with voxel world collision detection

## Testing Philosophy
- Unit tests embedded with `#[cfg(test)]`
- Standalone system demos in root directory
- Interactive 3D demos must show real graphics windows
- Performance benchmarks in `benches/`

## Performance Optimization

**Achieved Production-Ready Performance:**
- **Frustum Culling**: 92% efficiency - only renders visible chunks
- **Greedy Meshing**: 60-80% vertex reduction for optimal GPU utilization
- **Apple Silicon Optimization**: Native Metal rendering with unified memory support
- **Chunk Streaming**: Dynamic LOD and background chunk loading
- **GPU Acceleration**: Hardware-accelerated particle systems and lighting
- **Memory Management**: LRU caching and memory-mapped file I/O
- **Parallel Processing**: Rayon-based parallel chunk generation and physics

**Performance Commands:**
```bash
# Release build for performance testing
cargo build --release

# Performance profiling
RUST_LOG=info cargo run --release --bin robin_test

# Memory usage monitoring
cargo run --release --bin robin_test 2>&1 | grep -i memory
```

**CRITICAL**: AN ASCII TERMINAL-BASED DEMO IS _NOT_ A REAL DEMONSTRATION OF A 3D GAME ENGINE. NEVER, EVER, _EVER_ CONFUSE THAT. Always use real wgpu windows with Metal rendering.