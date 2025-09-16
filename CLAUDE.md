# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Robin is a comprehensive 2D/3D game engine built in Rust that has evolved into an advanced **Engineer Build Mode** system - an in-game development environment where players control an engineer character who can dynamically create and modify game worlds in real-time with interactive 3D graphics.

## Critical User Requirements

**IMPORTANT**: The user explicitly requires real 3D graphics windows with interactive navigation - NOT ASCII-based terminal demos:
- Real first-person perspective with WASD movement controls
- Fully textured environments with dynamic lighting
- Interactive terrain following and realistic movement physics
- Visible sun/moon in skybox with realistic day/night cycles
- Real wgpu-based graphics windows that appear on macOS

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
```

### Running 3D Graphics Demonstrations (REAL GRAPHICS)
```bash
# Full-featured 3D demo
cargo run --example robin_3d_showcase

# Simple 3D cube demo
cargo run --example simple_3d_demo

# Interactive voxel world (standalone compilation)
rustc voxel_world_demo.rs -o voxel_world_demo && ./voxel_world_demo

# Real 3D graphics window
rustc real_3d_window.rs -o real_3d_window && ./real_3d_window
```

### Running 2D/Particle Demos
```bash
# Magical demo with particles (default)
cargo run
cargo run magical

# Basic windowed demo
cargo run basic
```

### Testing Engineer Build Mode Systems (Standalone)
```bash
# These compile independently without full library
rustc simple_3d_playtest.rs -o simple_3d_playtest && ./simple_3d_playtest
rustc vehicle_test.rs -o vehicle_test && ./vehicle_test
rustc npc_ai_test.rs -o npc_ai_test && ./npc_ai_test
rustc world_construction_test.rs -o world_construction_test && ./world_construction_test
rustc story_simple_test.rs -o story_simple_test && ./story_simple_test
rustc advanced_tools_test.rs -o advanced_tools_test && ./advanced_tools_test
```

## Architecture Overview

### Dual Architecture Design

1. **Traditional 2D Game Engine** (`src/engine/`)
   - Core window management, input handling, game loop
   - WGPU-based 2D rendering with particles and animations
   - GameBuilder API for no-code development
   - Component-based scene management

2. **Engineer Build Mode System** (Phase 1 & 2 Complete)
   - 3D graphics with first-person controls
   - Voxel-based world construction
   - AI-assisted development tools
   - NPC behavior systems with social dynamics
   - Story and quest management
   - Vehicle physics and transportation

### Key Dependencies
- **wgpu 0.20**: WebGPU graphics API for all 3D rendering
- **winit 0.29**: Cross-platform windowing (critical for macOS)
- **cgmath + nalgebra**: 3D math transformations
- **smartcore + ndarray**: Machine learning for AI systems
- **tokio**: Async runtime
- **serde ecosystem**: Serialization

## Code Organization

### System Structure Pattern
```
src/engine/{system}/
├── mod.rs          # Public API
├── {system}.rs     # Core implementation
└── subsystems/     # Specialized components
```

### Critical Implementation Areas
- **3D Graphics**: `src/graphics/`, `voxel_world_demo.rs`, `real_3d_window.rs`
- **Character Physics**: `src/engine/character/character_physics.rs`, `engineer_controller.rs`
- **World Construction**: `src/engine/world/construction/`
- **AI Systems**: `src/engine/ai/`
- **GameBuilder API**: `src/engine/game_builder.rs`

## Current Development Status

**Phase 1**: Core Systems ✅ **COMPLETE**
- All 7 foundational systems implemented
- Working 3D graphics with real windows
- Character movement with terrain following
- Voxel-based world construction

**Phase 2**: Advanced Features ✅ **COMPLETE**
- Visual scripting and behavior trees
- Multiplayer collaboration
- Performance optimization (LOD, GPU acceleration)
- Advanced graphics (PBR, weather, particles)
- Audio and immersion systems

**Phase 3**: Polish and Distribution 🔄 **CURRENT**
- User interface modernization (NEXT)
- Asset pipeline enhancement
- Platform integration

## Known Issues

1. **Voxel Terrain Rendering**: Face generation only shows vertical edges instead of horizontal ground plane
   - Location: `voxel_world_demo.rs` in `get_visible_faces` function
   - Need to fix horizontal face culling logic

2. **Module Organization**: Some modules temporarily disabled in `src/engine/mod.rs`
   - Re-enable character, world, AI modules after fixing compilation

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

## Testing Philosophy
- Unit tests embedded with `#[cfg(test)]`
- Standalone system demos in root directory
- Interactive 3D demos must show real graphics windows
- Performance benchmarks in `benches/`

## Performance Considerations
- Use batch rendering for multiple objects
- Implement LOD for distant objects
- Stream voxel chunks as needed
- Use spatial hashing for collision detection
- Profile with `cargo build --release` for production testing