/// WGPU Clear Color Test - Phase 2 of Robin Engine Validation
///
/// This test validates WGPU surface rendering with animated clear colors
/// but no vertex buffers or complex geometry.

use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, ControlFlow},
    window::{WindowBuilder, Window},
    dpi::PhysicalSize,
};
use std::time::Instant;
use std::sync::Arc;

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: PhysicalSize<u32>,
    start_time: Instant,
    frame_count: u64,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        println!("🎨 Initializing WGPU...");

        // Create WGPU instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        println!("   ✅ WGPU instance created");

        // Create surface
        let surface = instance.create_surface(window).unwrap();
        println!("   ✅ Surface created");

        // Request adapter
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.expect("Failed to find suitable adapter");

        println!("   ✅ Adapter found: {:?}", adapter.get_info().name);
        println!("      Backend: {:?}", adapter.get_info().backend);
        println!("      Driver: {:?}", adapter.get_info().driver);

        // Create device and queue
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("WGPU Clear Test Device"),
            },
            None,
        ).await.expect("Failed to create device");

        println!("   ✅ Device and queue created");

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        println!("   ✅ Surface configured:");
        println!("      Format: {:?}", surface_format);
        println!("      Size: {}x{}", size.width, size.height);
        println!("      Present Mode: {:?}", config.present_mode);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            start_time: Instant::now(),
            frame_count: 0,
        }
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            println!("📐 Surface resized to {}x{}", new_size.width, new_size.height);
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.frame_count += 1;

        // Calculate animated color based on time
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let r = (elapsed.sin() * 0.5 + 0.5) as f64;
        let g = ((elapsed * 1.3).sin() * 0.5 + 0.5) as f64;
        let b = ((elapsed * 0.7).sin() * 0.5 + 0.5) as f64;

        // Get surface texture
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create command encoder
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Clear Color Encoder"),
        });

        // Begin render pass with animated clear color
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Color Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r, g, b, a: 1.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            // Render pass automatically ends when dropped
        }

        // Submit command buffer
        self.queue.submit(std::iter::once(encoder.finish()));

        // Present the frame
        output.present();

        // Log every 60 frames
        if self.frame_count % 60 == 0 {
            let fps = self.frame_count as f64 / elapsed;
            println!("🎬 Frame {} | FPS: {:.2} | Color: ({:.2}, {:.2}, {:.2})",
                     self.frame_count, fps, r, g, b);
        }

        Ok(())
    }
}

fn main() {
    println!("🚀 Robin Engine - WGPU Clear Color Test");
    println!("========================================");

    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");

    let window = Arc::new(WindowBuilder::new()
        .with_title("Robin Engine - WGPU Clear Color Test")
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_position(winit::dpi::PhysicalPosition::new(100, 100))
        .with_visible(true)
        .with_resizable(true)
        .build(&event_loop)
        .expect("Failed to create window"));

    println!("✅ Window created successfully!");

    // Force window to front on macOS
    window.set_visible(true);
    window.focus_window();
    window.request_redraw();

    // Create WGPU state
    let mut state = pollster::block_on(State::new(Arc::clone(&window)));
    println!("✅ WGPU initialized successfully!");

    println!("\n🎮 Controls:");
    println!("   ESC - Exit");
    println!("   R   - Force resize");
    println!("   F   - Toggle fullscreen");
    println!("\n🌈 Watch the colors cycle!");

    let mut last_fps_update = Instant::now();
    let mut is_fullscreen = false;

    // Run event loop
    let _ = event_loop.run(move |event, elwt| {
        elwt.set_control_flow(ControlFlow::Poll);

        match event {
            Event::WindowEvent { event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => {
                        println!("👋 Close requested - shutting down");
                        elwt.exit();
                    }
                    WindowEvent::Resized(physical_size) => {
                        state.resize(physical_size);
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        if event.state == winit::event::ElementState::Pressed {
                            match event.physical_key {
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::Escape) => {
                                    println!("🛑 ESC pressed - exiting");
                                    elwt.exit();
                                }
                                winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyR) => {
                                    println!("🔄 Forcing resize");
                                    state.resize(state.size);
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
                                _ => {}
                            }
                        }
                    }
                    WindowEvent::RedrawRequested => {
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => {
                                println!("⚠️  Surface lost, reconfiguring...");
                                state.resize(state.size);
                            }
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                println!("❌ Out of memory!");
                                elwt.exit();
                            }
                            Err(e) => {
                                println!("⚠️  Render error: {:?}", e);
                            }
                        }

                        // Update FPS counter
                        if last_fps_update.elapsed().as_secs() >= 1 {
                            last_fps_update = Instant::now();
                        }
                    }
                    _ => {}
                }
            }
            Event::AboutToWait => {
                // Request continuous redraws
                window.request_redraw();
                std::thread::sleep(std::time::Duration::from_millis(16));
            }
            Event::LoopExiting => {
                let runtime = state.start_time.elapsed().as_secs_f64();
                println!("🏁 Exiting - {} frames in {:.2}s ({:.2} FPS average)",
                         state.frame_count, runtime, state.frame_count as f64 / runtime);
            }
            _ => {}
        }
    });
}