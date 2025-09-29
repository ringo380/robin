/// Standalone minimal 3D triangle test
/// This is a completely independent test that doesn't depend on the Robin engine

use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

fn main() {
    println!("🎮 Starting minimal wgpu test...");

    // Test 1: Can we create an event loop?
    let event_loop = match EventLoop::new() {
        Ok(el) => {
            println!("✅ Event loop created successfully");
            el
        }
        Err(e) => {
            println!("❌ Failed to create event loop: {}", e);
            return;
        }
    };

    // Test 2: Can we create a window?
    let window = match WindowBuilder::new()
        .with_title("🔥 Minimal wgpu Test")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop) {
        Ok(win) => {
            println!("✅ Window created successfully");
            win
        }
        Err(e) => {
            println!("❌ Failed to create window: {}", e);
            return;
        }
    };

    println!("🎯 If you can see this message, basic windowing is working!");
    println!("📝 Press ESC to exit or close the window");

    // Simple event loop - just keep the window open
    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            physical_key: PhysicalKey::Code(KeyCode::Escape),
                            ..
                        },
                    ..
                } => {
                    println!("👋 Goodbye!");
                    target.exit()
                },
                _ => {}
            },
            _ => {}
        }
    }).unwrap();
}