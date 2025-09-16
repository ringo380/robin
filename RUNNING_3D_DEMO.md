# Running the Robin Engine 3D Graphics Showcase

## Overview
The Robin Engine 3D Showcase (`robin_3d_showcase`) is a comprehensive demonstration of the engine's real-time 3D graphics capabilities, featuring:

- **Real-time PBR rendering** with metallic/roughness workflow
- **Dynamic lighting system** with directional, point, and spot lights
- **GPU-accelerated particle system** with up to 10,000 particles
- **Interactive first-person controls** with physics simulation
- **Instanced rendering** for efficient drawing of many objects
- **Post-processing effects** including fog and tone mapping
- **Interactive building system** with 5 material types

## Building and Running

### First-time Setup
```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone the repository
git clone <repository-url>
cd robin

# Build the demo (first build will take 2-3 minutes to compile dependencies)
cargo build --release --example robin_3d_showcase
```

### Running the Demo
```bash
# Run in debug mode (slower, with debug info)
cargo run --example robin_3d_showcase

# Run in release mode (optimized, 60+ FPS)
cargo run --release --example robin_3d_showcase
```

## Controls

### Movement
- **WASD** - Move forward/backward/left/right
- **Space** - Jump
- **Shift** - Sprint (2x speed)
- **Mouse** - Look around (first-person camera)

### Building & Interaction
- **Left Click** - Place block at cursor position
- **Right Click** - Remove block
- **E** - Interact with objects
- **1-5** - Select material type:
  - 1: Stone (gray, rough)
  - 2: Metal (reflective, smooth)
  - 3: Wood (brown, natural)
  - 4: Glass (transparent, smooth)
  - 5: Energy (glowing, emissive)

### Visual Effects
- **Q** - Toggle wireframe mode
- **F** - Toggle flashlight
- **G** - Spawn particle burst (100 particles)
- **Tab** - Toggle debug UI

### System
- **Escape** - Release mouse / Exit application

## Features Demonstrated

### 1. PBR Materials System
The demo showcases 5 different material types, each with unique properties:
- Albedo color
- Metallic value (0-1)
- Roughness value (0-1)
- Emissive intensity

### 2. Dynamic Lighting
- **Directional light** (sun) casting from above
- **Animated point lights** with color and intensity variations
- **Spot light** (flashlight) that follows camera direction
- Real-time light calculations with attenuation

### 3. Particle System
- GPU-optimized particle rendering
- Physics simulation with gravity
- Billboard rendering (particles always face camera)
- Soft particles with depth testing
- Multiple particle effects:
  - Block placement dust
  - Block destruction debris
  - Magic particle bursts

### 4. Interactive World Building
- Voxel-based construction system
- Grid-aligned block placement
- Ray-casting for block selection
- Material painting system
- Real-time mesh updates

### 5. Physics Simulation
- Character controller with gravity and jumping
- Dynamic objects with bouncing physics
- Velocity-based movement system
- Ground collision detection

## Performance Metrics

The demo displays real-time performance metrics in the console:
- **FPS** - Frames per second
- **Objects** - Number of rendered objects
- **Particles** - Active particle count

Target performance (on modern hardware):
- 60+ FPS at 1920x1080
- 1000+ objects rendered simultaneously
- 10,000 particles without frame drops

## Technical Implementation

### Graphics Pipeline
- **Renderer**: wgpu 0.20 (WebGPU implementation)
- **Shading**: WGSL shaders with PBR lighting model
- **Instancing**: Efficient batch rendering of similar objects
- **Depth Testing**: 32-bit float depth buffer
- **Anti-aliasing**: MSAA support (configurable)

### Shader System
Two main shader programs:
1. **main_3d.wgsl** - PBR material rendering with instancing
2. **particles_3d.wgsl** - Billboard particle rendering with soft edges

### Scene Composition
The demo creates an initial scene with:
- 441 ground tiles (21x21 grid with alternating materials)
- 4 energy pillars at cardinal points
- 10 floating dynamic objects with physics
- Dynamic particle effects based on player actions

## Troubleshooting

### Build Issues
If the build fails:
1. Ensure Rust is updated: `rustup update`
2. Clean build cache: `cargo clean`
3. Check wgpu compatibility with your GPU

### Performance Issues
If FPS is low:
1. Run in release mode: `cargo run --release --example robin_3d_showcase`
2. Reduce particle count (modify MAX_PARTICLES constant)
3. Check GPU drivers are up to date

### Graphics Issues
If rendering looks incorrect:
1. Update graphics drivers
2. Check wgpu backend selection (Vulkan/Metal/DX12)
3. Verify shader compilation in console output

## System Requirements

### Minimum
- CPU: Dual-core 2.0 GHz
- RAM: 4 GB
- GPU: DirectX 11 / Metal / Vulkan compatible
- OS: Windows 10, macOS 10.15, Linux (Ubuntu 20.04)

### Recommended
- CPU: Quad-core 3.0 GHz
- RAM: 8 GB
- GPU: Dedicated graphics with 2GB VRAM
- OS: Latest versions

## Next Steps

After running the demo, explore:
1. Modify material properties in the code
2. Add new mesh types (sphere, cylinder, pyramid)
3. Implement shadow mapping
4. Add texture loading support
5. Create custom particle effects
6. Build more complex scenes

The demo serves as a foundation for building full 3D games with Robin Engine's graphics capabilities.