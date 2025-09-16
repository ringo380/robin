# Robin Engine Hot Reload Demo

This demo showcases Robin Engine's comprehensive asset hot-reloading system, allowing you to see changes to textures, audio files, and configuration files in real-time without restarting your game.

## Quick Start

1. **Run the demo:**
   ```bash
   cargo run --example hot_reload_demo
   ```

2. **In another terminal, run the automated tester:**
   ```bash
   python3 examples/hot_reload_tester.py
   ```

3. **Or manually edit files in `examples/assets/` while the demo runs**

## What You'll See

### Automatic Features
- 🔍 **File Watching**: Automatically detects changes to asset files
- 🔄 **Auto-Reload**: Reloads assets when files are modified
- 📊 **Statistics**: Real-time stats about reload performance
- 🧹 **Cleanup**: Handles deleted files gracefully
- ⚡ **Rate Limiting**: Prevents excessive reloads during rapid changes

### Interactive Commands
- Type `r` + Enter: Force reload the player sprite
- Type `s` + Enter: Show current hot reload statistics
- Type `q` + Enter: Quit the demo

## Asset Types Supported

### 🎨 Textures
- **Formats**: PNG, JPG, JPEG, GIF, BMP, TGA
- **Location**: `examples/assets/textures/`
- **Test**: Replace `player.png` with different images

### 🎵 Audio
- **Formats**: WAV, MP3, OGG, FLAC
- **Location**: `examples/assets/audio/`
- **Test**: Replace `bgm.ogg` with different audio files

### ⚙️ Configuration
- **Formats**: JSON, TOML, YAML
- **Location**: `examples/assets/config/`
- **Test**: Modify `settings.json` values

## Demo File Structure

```
examples/
├── hot_reload_demo.rs          # Main demo application
├── hot_reload_tester.py        # Automated testing script
├── README_HOT_RELOAD.md        # This file
└── assets/                     # Auto-generated demo assets
    ├── textures/
    │   └── player.png
    ├── audio/
    │   └── bgm.ogg
    └── config/
        └── settings.json
```

## Testing Scenarios

### 1. Configuration Changes
Edit `examples/assets/config/settings.json`:
```json
{
    "player_speed": 7.5,
    "jump_height": 12.0,
    "gravity": 9.8,
    "version": "1.1.0",
    "debug_mode": true
}
```

### 2. Texture Replacement
Replace `examples/assets/textures/player.png` with any PNG image. You'll see reload events in the console.

### 3. File Deletion/Recreation
- Delete a watched file → See deletion event
- Recreate it → See creation and reload events

### 4. Rapid Changes
The Python tester script makes rapid changes to test the rate limiting and batching features.

## Performance Features

### Rate Limiting
- Prevents excessive I/O by limiting file checks to every 50ms
- Batches multiple changes that occur rapidly

### Thread Safety
- File watching runs in background thread
- Thread-safe asset registry with RwLock
- Safe concurrent access to asset data

### Memory Management
- Tracks asset memory usage
- Provides cleanup for deleted files
- Statistics for monitoring performance

## Real-World Usage

In your games, integrate hot reload like this:

```rust
use robin::engine::prelude::*;

let mut game = GameBuilder::new()
    .enable_hot_reload(true)
    .hot_reload_config(|config| {
        config
            .watch_delay(Duration::from_millis(100))
            .base_path("assets")
    })
    .build()?;

// Register specific assets
game.register_asset("player", "assets/sprites/player.png")
    .register_asset("bgm", "assets/music/theme.ogg");

// Add custom reload callbacks
game.add_reload_callback("player", Box::new(|event| {
    if let HotReloadEvent::AssetModified { asset_id, .. } = event {
        println!("Player sprite updated: {}", asset_id);
        // Update your game's texture references here
    }
}));

// Start the hot reload system
game.start_hot_reload();

// In your game loop
loop {
    game.update(delta_time); // Processes hot reload events
    // Your game logic here
}
```

## Troubleshooting

### Assets Not Reloading?
- Check file permissions
- Verify file paths are correct
- Ensure hot reload is enabled: `.enable_hot_reload(true)`
- Check console for error messages

### Performance Issues?
- Increase watch delay: `.watch_delay(Duration::from_millis(200))`
- Reduce number of watched assets
- Check statistics with `get_hot_reload_stats()`

### Files Not Found?
The demo automatically creates dummy assets. If you see file not found errors:
1. Delete the `examples/assets/` directory
2. Run the demo again to recreate assets

## Architecture Overview

The hot reload system consists of:

1. **AssetWatcher**: Cross-platform file system monitoring
2. **AssetRegistry**: Centralized asset storage and metadata
3. **HotReloadSystem**: Coordination and event processing
4. **GameBuilder Integration**: No-code API for easy use

This creates a robust, production-ready hot reload system that enhances development workflow significantly.