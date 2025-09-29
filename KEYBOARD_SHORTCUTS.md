# Robin Engine - Keyboard Shortcuts Reference

Complete guide to all keyboard shortcuts and controls in the Robin 3D Voxel Game Engine.

## 🎮 Movement & Camera Controls

### Basic Movement
- **WASD** - Move camera horizontally
  - **W** - Move forward
  - **A** - Move left (strafe)
  - **S** - Move backward
  - **D** - Move right (strafe)
- **Space** - Jump (when character is grounded)
- **Mouse Movement** - Look around (first-person view)
- **Mouse Scroll** - Zoom in/out

### Camera Rotation (Alternative)
- **Arrow Keys** - Rotate camera orientation
  - **Left Arrow** - Rotate left
  - **Right Arrow** - Rotate right
  - **Up Arrow** - Look up
  - **Down Arrow** - Look down

## 🏗️ Build Mode System

### Mode Selection
- **M** or **E** - Cycle through build modes:
  - **Build Mode** - Place and remove voxels
  - **Test Mode** - Test environment and physics
  - **Play Mode** - Interactive gameplay mode

### Voxel Interaction
- **Left Click** - Place voxel at cursor location
- **Right Click** - Remove voxel at cursor location

## 🎨 Material Selection (Number Keys)

- **1** - Stone (gray, solid building material)
- **2** - Dirt (brown, natural terrain)
- **3** - Grass (green, surface vegetation)
- **4** - Sand (tan, granular material)
- **5** - Wood (brown, construction material)
- **6** - Glass (transparent, decorative)
- **7** - Metal (metallic, industrial)
- **8** - Water (blue, liquid simulation)
- **9** - Obsidian (dark, volcanic material)

## 🔧 Templates & Construction Tools

### Template Controls
- **T** - Cycle through available building templates
- **R** - Rotate current template (90-degree increments)

### History & Undo System
- **Ctrl+Z** - Undo last action
- **Ctrl+Y** - Redo last undone action

## 🌍 World Management

### Save/Load System
- **Ctrl+S** - Save current world state
- **Ctrl+L** - List and load saved worlds (auto-loads most recent)

## 📋 Interface & System Controls

### Help & UI
- **H** - Toggle comprehensive help overlay
- **ESC** - Close overlays / Exit application
- **F1** - Toggle legacy help display (console output)
- **F2** - Toggle UI overlay visibility

## 🎯 Advanced Features

### Physics Integration
The engine includes realistic physics simulation:
- Character movement follows terrain
- Jumping only works when grounded
- Particle effects on voxel placement/removal

### Performance Optimization
- **Frustum Culling**: 92% rendering efficiency
- **Greedy Meshing**: 60-80% vertex reduction
- **LOD System**: Dynamic level-of-detail for large worlds
- **Chunk Streaming**: Background loading of world sections

## 📱 Visual HUD Features

The engine displays real-time information:
- **Current Mode**: Color-coded build mode indicator
- **Active Material**: Visual material selection display
- **Template Info**: Current template and rotation
- **Performance**: Real-time FPS with color-coded indicators
  - Green: 60+ FPS (excellent)
  - Orange: 30-59 FPS (good)
  - Red: <30 FPS (needs optimization)
- **History Status**: Undo/redo availability indicators

## 🚀 Getting Started

1. **Launch** Robin Engine (`cargo run --bin robin`)
2. **Move** around with WASD keys
3. **Look** around with mouse
4. **Press H** to view in-game help
5. **Press M/E** to enter Build Mode
6. **Select materials** with number keys 1-9
7. **Build** with left-click, **remove** with right-click
8. **Save your work** with Ctrl+S

## 🛠️ Engine Architecture

Robin is built on:
- **Rust** - Memory-safe systems programming
- **wgpu** - Modern graphics API (Metal on macOS)
- **winit** - Cross-platform windowing
- **rapier3d** - Physics simulation
- **cgmath** - 3D mathematics

---

*Robin Engine v2.0 - Advanced 3D Voxel Game Engine*
*Built for educational technology and creative construction*