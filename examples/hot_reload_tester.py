#!/usr/bin/env python3
"""
Hot Reload Tester Script
========================

This script helps test the Robin Engine's hot reload system by automatically
making changes to assets while the demo is running.

Usage: python3 examples/hot_reload_tester.py
"""

import os
import time
import json
import random
import shutil
from pathlib import Path

def main():
    print("🧪 Robin Hot Reload Tester")
    print("==========================")
    print()
    
    assets_dir = Path("examples/assets")
    
    if not assets_dir.exists():
        print("❌ Assets directory not found. Please run the hot_reload_demo first.")
        return
    
    print("🎯 This script will automatically modify assets to test hot reloading.")
    print("   Make sure the hot_reload_demo is running in another terminal!")
    print()
    
    # Wait for user confirmation
    input("Press Enter to start the automated test sequence...")
    print()
    
    # Test sequence
    test_config_changes(assets_dir)
    time.sleep(2)
    
    test_texture_changes(assets_dir)
    time.sleep(2)
    
    test_file_deletion_and_recreation(assets_dir)
    time.sleep(2)
    
    test_rapid_changes(assets_dir)
    
    print("✅ Hot reload testing complete!")
    print("   Check the demo console for reload events.")

def test_config_changes(assets_dir):
    """Test configuration file hot reloading"""
    print("🔧 Testing config file changes...")
    
    config_file = assets_dir / "config" / "settings.json"
    
    # Make several config changes
    configs = [
        {
            "player_speed": 7.5,
            "jump_height": 12.0,
            "gravity": 9.8,
            "version": "1.0.1",
            "debug_mode": True,
            "test_iteration": 1
        },
        {
            "player_speed": 3.0,
            "jump_height": 15.0,
            "gravity": 12.0,
            "version": "1.0.2",
            "debug_mode": False,
            "test_iteration": 2,
            "new_feature": "hot_reload_test"
        },
        {
            "player_speed": 5.0,
            "jump_height": 10.0,
            "gravity": 9.8,
            "version": "1.0.3",
            "debug_mode": True,
            "test_iteration": 3,
            "difficulty": "normal"
        }
    ]
    
    for i, config in enumerate(configs):
        print(f"  📝 Writing config change {i+1}...")
        with open(config_file, 'w') as f:
            json.dump(config, f, indent=2)
        time.sleep(1.5)
    
    print("✅ Config change test completed")

def test_texture_changes(assets_dir):
    """Test texture file hot reloading by replacing the file"""
    print("🎨 Testing texture changes...")
    
    texture_file = assets_dir / "textures" / "player.png"
    
    # Create different colored "textures" (simple pixel data variations)
    textures = [
        create_colored_png(255, 0, 0),    # Red
        create_colored_png(0, 255, 0),    # Green
        create_colored_png(0, 0, 255),    # Blue
        create_colored_png(255, 255, 0),  # Yellow
    ]
    
    colors = ["red", "green", "blue", "yellow"]
    
    for i, (texture_data, color) in enumerate(zip(textures, colors)):
        print(f"  🖼️  Changing texture to {color}...")
        with open(texture_file, 'wb') as f:
            f.write(texture_data)
        time.sleep(1.5)
    
    print("✅ Texture change test completed")

def test_file_deletion_and_recreation(assets_dir):
    """Test file deletion and recreation scenarios"""
    print("🗑️  Testing file deletion and recreation...")
    
    test_file = assets_dir / "textures" / "temp_test.png"
    
    # Create a test file
    print("  📁 Creating temporary test file...")
    with open(test_file, 'wb') as f:
        f.write(create_colored_png(128, 128, 128))  # Gray
    time.sleep(1)
    
    # Delete it
    print("  🗑️  Deleting test file...")
    test_file.unlink()
    time.sleep(1)
    
    # Recreate it
    print("  📁 Recreating test file...")
    with open(test_file, 'wb') as f:
        f.write(create_colored_png(255, 128, 0))  # Orange
    time.sleep(1)
    
    # Clean up
    test_file.unlink()
    print("✅ File deletion/recreation test completed")

def test_rapid_changes(assets_dir):
    """Test rapid successive changes to stress-test the system"""
    print("⚡ Testing rapid changes (stress test)...")
    
    config_file = assets_dir / "config" / "settings.json"
    
    for i in range(10):
        config = {
            "player_speed": random.uniform(1.0, 10.0),
            "jump_height": random.uniform(5.0, 20.0),
            "gravity": random.uniform(5.0, 15.0),
            "version": f"stress-test-{i}",
            "debug_mode": random.choice([True, False]),
            "rapid_change_id": i,
            "timestamp": time.time()
        }
        
        print(f"  ⚡ Rapid change {i+1}/10...")
        with open(config_file, 'w') as f:
            json.dump(config, f, indent=2)
        time.sleep(0.2)  # Very fast changes
    
    print("✅ Rapid changes test completed")

def create_colored_png(r, g, b):
    """Create a simple 2x2 colored PNG"""
    # This is a more complex PNG than the demo, with actual color data
    png_header = bytes([
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A  # PNG signature
    ])
    
    # IHDR chunk for 2x2 image
    ihdr = bytes([
        0x00, 0x00, 0x00, 0x0D,  # Length
        0x49, 0x48, 0x44, 0x52,  # "IHDR"
        0x00, 0x00, 0x00, 0x02,  # Width: 2
        0x00, 0x00, 0x00, 0x02,  # Height: 2
        0x08, 0x02, 0x00, 0x00, 0x00,  # Bit depth: 8, Color type: 2 (RGB), Compression: 0, Filter: 0, Interlace: 0
    ])
    
    # Simple CRC calculation (simplified)
    ihdr_crc = bytes([0x7D, 0xD2, 0x87, 0x21])  # Pre-calculated for 2x2 RGB
    
    # IDAT chunk with pixel data (4 pixels, 3 bytes each)
    pixel_data = bytes([
        0x00,  # Filter type for first row
        r, g, b,  # Pixel 1
        r, g, b,  # Pixel 2
        0x00,  # Filter type for second row  
        r, g, b,  # Pixel 3
        r, g, b,  # Pixel 4
    ])
    
    # Compress the pixel data (simplified - using raw data with minimal compression)
    import zlib
    compressed_data = zlib.compress(pixel_data)
    
    idat_length = len(compressed_data).to_bytes(4, 'big')
    idat_header = b'IDAT'
    idat_crc = zlib.crc32(idat_header + compressed_data).to_bytes(4, 'big')
    
    # IEND chunk
    iend = bytes([
        0x00, 0x00, 0x00, 0x00,  # Length: 0
        0x49, 0x45, 0x4E, 0x44,  # "IEND"
        0xAE, 0x42, 0x60, 0x82   # CRC
    ])
    
    return png_header + ihdr + ihdr_crc + idat_length + idat_header + compressed_data + idat_crc + iend

if __name__ == "__main__":
    main()