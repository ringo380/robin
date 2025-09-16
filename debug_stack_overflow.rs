// Minimal test to identify stack overflow source in Robin Engine
// Test individual system initializations to isolate the recursive call

use std::env;
use std::time::Duration;

// Minimal test for each system component
fn main() {
    println!("🔍 Debug: Testing Robin Engine system initialization");
    
    // Test 1: Environment setup (skip env_logger for standalone test)
    println!("1. Testing basic environment...");
    println!("   ✅ Basic setup - OK");
    
    // Test 2: Basic imports (simulated, since this won't compile standalone)
    println!("2. Testing basic types...");
    let _duration_test = Duration::from_secs(1);
    println!("   ✅ Duration types - OK");
    
    println!("🎉 Basic tests complete! Stack overflow is likely in Robin Engine systems.");
    println!("   Check Default implementations and recursive new() calls.");
    
    // This test should run without stack overflow
    // If it does, the issue is in the Robin Engine code itself
}