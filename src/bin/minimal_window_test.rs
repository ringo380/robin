/// Minimal Window Test - Phase 1 of Robin Engine Validation
///
/// This test establishes baseline window functionality on macOS
/// with proper positioning, visibility, and event handling.

use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
    dpi::PhysicalSize,
};
use std::time::Instant;

fn main() {
    println!("🚀 Robin Engine - Minimal Window Test");
    println!("=====================================");

    // Create event loop
    let event_loop = EventLoop::new().expect("Failed to create event loop");

    // Create window with explicit configuration for macOS
    let window = WindowBuilder::new()
        .with_title("Robin Engine - Minimal Window Test")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_position(winit::dpi::PhysicalPosition::new(100, 100))
        .with_visible(true)
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create window");

    // Log window properties
    println!("✅ Window created successfully!");
    println!("   Title: {}", window.title());
    println!("   Size: {:?}", window.inner_size());
    println!("   Position: {:?}", window.outer_position().unwrap_or_default());
    println!("   Scale Factor: {}", window.scale_factor());
    println!("   Monitor: {:?}", window.current_monitor().map(|m| m.name()));

    // Force window to front on macOS
    window.set_visible(true);
    window.focus_window();
    window.request_redraw();

    println!("\n📊 Window Status:");
    println!("   Visible: {:?}", window.is_visible());
    println!("   Focused: {:?}", window.has_focus());
    println!("   Minimized: {:?}", window.is_minimized());

    println!("\n🎮 Controls:");
    println!("   ESC - Exit");
    println!("   R   - Request redraw");
    println!("   F   - Toggle fullscreen");
    println!("   M   - Minimize window");

    let start_time = Instant::now();
    let mut frame_count = 0u64;
    let mut last_fps_update = Instant::now();
    let mut is_fullscreen = false;

    // Run event loop with proper control flow
    let _ = event_loop.run(move |event, elwt| {
        // Use Poll control flow for continuous updates
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("👋 Close requested - shutting down gracefully");
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        println!("📐 Window resized to {}x{}",
                                 physical_size.width, physical_size.height);
                    }
                    WindowEvent::Moved(physical_position) => {
                        println!("📍 Window moved to ({}, {})",
                                 physical_position.x, physical_position.y);
                    }
                    WindowEvent::Focused(focused) => {
                        println!("🎯 Window focus changed: {}",
                                 if focused { "Focused" } else { "Unfocused" });
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == winit::event::ElementState::Pressed {
                            match event.physical_key {
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => {
                                    println!("🛑 ESC pressed - exiting");
                                    elwt.exit();
                                }
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyR) => {
                                    println!("🔄 Requesting redraw");
                                    window.request_redraw();
                                }
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyF) => {
                                    is_fullscreen = !is_fullscreen;
                                    println!("🖥️  Toggling fullscreen: {}", is_fullscreen);
                                    window.set_fullscreen(if is_fullscreen {
                                        Some(winit::window::Fullscreen::Borderless(None))
                                    } else {
                                        None
                                    });
                                }
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyM) => {
                                    println!("📦 Minimizing window");
                                    window.set_minimized(true);
                                }
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        frame_count += 1;

                        // Update FPS counter every second
                        if last_fps_update.elapsed().as_secs() >= 1 {
                            let fps = frame_count as f64 / start_time.elapsed().as_secs_f64();
                            println!("🎬 Frame {} | FPS: {:.2} | Runtime: {:.1}s",
                                     frame_count, fps, start_time.elapsed().as_secs_f64());
                            last_fps_update = Instant::now();
                        }

                        // In a real application, rendering would happen here
                        // For this test, we just validate the window is responsive
                    }
                    WindowEvent::Occluded(occluded) => {
                        println!("👁️  Window occluded: {}", occluded);
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // Request continuous redraws for animation
                window.request_redraw();

                // Small delay to avoid excessive CPU usage
                std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
            }
            Event::LoopExiting => {
                println!("🏁 Event loop exiting - cleanup complete");
                let runtime = start_time.elapsed().as_secs_f64();
                println!("📈 Final stats: {} frames in {:.2}s ({:.2} FPS average)",
                         frame_count, runtime, frame_count as f64 / runtime);
            }
            _ => {}
        }
    });
}