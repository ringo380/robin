# Robin Game Engine

A 2D/isometric game engine built from scratch in Rust, designed for efficiency, compatibility, accessibility, flexibility, and ease-of-use.

## Features

### 🎨 Advanced Visual Systems
- **Real-time 3D Rendering** - Modern PBR materials with metallic/roughness workflow
- **Dynamic Lighting** - Directional, point, and spot lights with real-time shadows
- **GPU Particle System** - Hardware-accelerated effects with up to 10k particles
- **Post-Processing** - Bloom, fog, tone mapping, and gamma correction
- **Instanced Rendering** - Efficient drawing of thousands of objects
- **Dynamic 2D Lighting** - Real-time point lights with color, intensity, and radius
- **Smooth Animations** - 12+ easing types (bounce, elastic, back, etc.)
- **Modern Shaders** - WGSL shaders with advanced lighting calculations

### 🚀 No-Code Friendly API
- **GameBuilder** - High-level API for easy game creation
- **Preset Effects** - One-line campfires, explosions, portals, treasure pickups
- **Scene Presets** - Quick dungeon, night, and magical forest setups
- **Animation Helpers** - Simple fade, move, bounce, and pulse methods

### 🏗️ Core Engine Architecture  
- **Cross-platform windowing** with winit
- **WebGPU rendering** with wgpu for modern graphics
- **Component-based** game objects and scene management
- **Type-safe asset management** with hot-reload ready design
- **Input system** supporting keyboard and mouse
- **Math utilities** with cgmath integration for 2D/isometric games

## Quick Start

### Prerequisites
- Rust 1.70+ (2021 edition)
- Git

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd robin
```

2. Run the magical demo (default):
```bash
cargo run
# OR specifically:
cargo run magical
```

3. Try the basic windowed demo:
```bash
cargo run basic
```

4. Experience the 3D graphics showcase:
```bash
# Full-featured 3D demo with PBR materials and interactive building
cargo run --example robin_3d_showcase

# Simple 3D cube demo (faster compilation)
cargo run --example simple_3d_demo

# Existing 3D engineer demo
cargo run --example engineer_3d_demo
```

The magical demo showcases particle effects, dynamic lighting, and smooth animations, while the 3D demos demonstrate real-time rendering, physics, and interactive gameplay!

## Project Structure

```
src/
├── engine/                    # Core engine modules
│   ├── core/                  # Engine initialization and main loop
│   ├── graphics/              # Advanced rendering systems
│   │   ├── renderer.rs        # WGPU renderer with lighting
│   │   ├── particles.rs       # Particle system with presets
│   │   ├── camera.rs          # 2D/isometric camera system
│   │   └── shaders/           # WGSL shader files
│   ├── animation.rs           # Smooth animation system
│   ├── game_builder.rs        # No-code friendly API
│   ├── input/                 # Modern input handling
│   ├── audio/                 # Audio system (expandable)
│   ├── math/                  # Math utilities and types  
│   ├── assets/                # Asset loading and management
│   └── scene/                 # Component-based scene graph
├── examples/                  # Demo applications
│   ├── basic_window.rs        # Simple windowed demo
│   └── magical_demo.rs        # Visual effects showcase
└── main.rs                    # Entry point with demo selection
```

## Engine Modules

### Core (`engine::core`)
- Main engine loop
- Window management with winit
- Event handling

### Graphics (`engine::graphics`)
- WGPU-based rendering
- Camera system for 2D/isometric views
- Texture and shader management
- Future: Sprite batching, particle systems

### Input (`engine::input`)
- Keyboard and mouse input handling
- Modern winit API integration
- Future: Gamepad support, input mapping

### Audio (`engine::audio`)
- Basic audio management interface
- Future: Spatial audio, music/sfx separation

### Math (`engine::math`)
- Vector and matrix types (Vec2, Vec3, Mat4)
- Transform utilities for 2D/isometric rendering
- Integration with cgmath library

### Assets (`engine::assets`)
- Generic asset loading trait
- Type-safe asset management
- Future: Hot-reloading, asset bundling

### Scene (`engine::scene`)
- Component-based game objects
- Transform hierarchy
- Scene management

## 🎮 Usage Examples

### No-Code Game Creation

```rust
use robin::GameBuilder;

let mut game = GameBuilder::new();

// Create a magical forest scene
game.setup_magical_forest(&[(100.0, 200.0), (300.0, 150.0)]);

// Add a campfire with realistic effects
game.create_campfire(400.0, 300.0);

// Create treasure with pickup effects  
let treasure_effects = game.create_treasure_pickup(200.0, 100.0);

// Animate objects smoothly
game.move_object("player", 0.0, 0.0, 100.0, 50.0, 2.0)  // 2 second movement
    .fade_in("ui_element", 1.0)                          // 1 second fade in
    .bounce_object("coin", 1.0, 1.5, 0.5);              // Bouncy scale effect

// Update all systems (call every frame)
let delta_time = 0.016; // ~60 FPS  
let animation_updates = game.update(delta_time);
```

### Custom Visual Effects

```rust
// Create explosion at mouse click
game.create_explosion(mouse_x, mouse_y);

// Magical portal with swirling effects
let portal_effects = game.create_portal(300.0, 200.0);

// Colorful fireworks
game.create_fireworks(500.0, 100.0, (1.0, 0.2, 0.8)); // Pink fireworks

// Dynamic lighting
game.add_torch_light(150.0, 150.0)              // Flickering torch
    .add_magic_light(300.0, 100.0)              // Blue magical glow
    .add_light(200.0, 200.0, (1.0, 0.8, 0.2), 0.8); // Custom warm light
```

### Advanced Animations

```rust
// Smooth movement with easing
game.move_object("character", 0.0, 0.0, 200.0, 100.0, 3.0);

// Color pulsing effect
game.pulse_color("gem", 
    (1.0, 0.2, 0.2, 1.0),    // Red
    (1.0, 0.8, 0.2, 1.0),    // Gold  
    1.5);                     // 1.5 second cycle

// Elastic scaling for impact
game.elastic_scale("button", 1.0, 1.3, 0.8);

// Continuous spinning
game.spin_object("wheel", 2.0); // 2 rotations per second
```

## Dependencies

- **wgpu 0.20**: Modern WebGPU graphics API for high-performance rendering
- **winit 0.29**: Cross-platform windowing and event handling
- **cgmath 0.18**: Linear algebra and math utilities for 2D/3D transforms
- **bytemuck**: Zero-copy type conversions for GPU data
- **tokio**: Async runtime for smooth engine operations
- **rand**: Random number generation for particle effects
- **image**: Future texture and sprite loading
- **env_logger**: Development logging and debugging

## Development

### Building
```bash
cargo build
```

### Running Examples
```bash
cargo run --example basic_window
```

### Testing
```bash
cargo test
```

### Benchmarks
```bash
cargo bench
```

## Roadmap

### Phase 1: Foundation ✅
- [x] Basic project structure  
- [x] Core engine loop
- [x] Window management
- [x] Input system
- [x] Math utilities

### Phase 2: Advanced Graphics ✅
- [x] WGPU renderer integration
- [x] Dynamic lighting system (64 concurrent lights)
- [x] Particle effects with presets
- [x] Smooth animation system (12+ easing types)
- [x] Modern WGSL shader pipeline
- [x] Batch rendering system

### Phase 3: Developer Experience ✅ 
- [x] No-code GameBuilder API
- [x] Visual effects presets (explosions, magic, fire, fog)
- [x] Animation helpers (fade, move, bounce, pulse)
- [x] Scene setup presets (dungeon, forest, night)
- [x] Comprehensive examples and demos

### Phase 4: Enhanced Features (In Progress)
- [ ] Texture and sprite system integration
- [ ] Physics integration (2D rigid bodies)
- [ ] Audio system expansion  
- [ ] UI system with animations
- [ ] Asset hot-reloading
- [ ] Scene serialization/loading

### Phase 5: Production Tools
- [ ] Visual editor interface
- [ ] Asset pipeline with optimization
- [ ] Performance profiling and optimization
- [ ] Export to multiple platforms
- [ ] Plugin/mod system

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

Licensed under either of:
- MIT license
- Apache License, Version 2.0

at your option.