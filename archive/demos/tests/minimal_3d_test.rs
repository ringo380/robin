// Minimal 3D Graphics Test for Robin Engine
// A standalone test to verify wgpu graphics initialization works

use winit::{
    event::*,
    event_loop::{EventLoop, ControlFlow},
    window::WindowBuilder,
};

fn main() {
    println!("🚀 Minimal 3D Graphics Test");
    println!("============================");
    
    // Create event loop and window
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_title("Robin Engine - Minimal 3D Test")
        .with_inner_size(winit::dpi::LogicalSize::new(800, 600))
        .build(&event_loop)
        .unwrap();
    
    println!("✅ Window created successfully");
    
    // Initialize WGPU
    let rt = tokio::runtime::Runtime::new().unwrap();
    let graphics_state = rt.block_on(async {
        init_wgpu(&window).await
    });
    
    match graphics_state {
        Ok(_) => {
            println!("✅ WGPU initialized successfully");
            println!("🎮 Graphics system is working!");
            println!("Press Escape or close window to exit");
        },
        Err(e) => {
            println!("❌ WGPU initialization failed: {}", e);
            return;
        }
    }
    
    // Simple event loop
    let mut frame_count = 0;
    event_loop.run(move |event, target, control_flow| {
        *control_flow = ControlFlow::Wait;
        
        match event {
            Event::WindowEvent { ref event, window_id } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested 
                    | WindowEvent::KeyboardInput {
                        input: KeyboardInput {
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            state: ElementState::Pressed,
                            ..
                        },
                        ..
                    } => {
                        println!("👋 Exiting after {} frames", frame_count);
                        *control_flow = ControlFlow::Exit;
                    },
                    
                    WindowEvent::RedrawRequested => {
                        frame_count += 1;
                        if frame_count % 60 == 0 {
                            println!("💫 Rendered {} frames", frame_count);
                        }
                        // Basic clear screen would go here
                        window.request_redraw();
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                frame_count += 1;
                if frame_count % 60 == 0 {
                    println!("💫 Rendered {} frames", frame_count);
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

async fn init_wgpu(window: &winit::window::Window) -> Result<(), String> {
    println!("🔧 Initializing WGPU...");
    
    // Create WGPU instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        dx12_shader_compiler: Default::default(),
    });
    
    println!("🔧 WGPU instance created");
    
    // Create surface
    let surface = unsafe { 
        instance.create_surface(window) 
    }.map_err(|e| format!("Failed to create surface: {}", e))?;
    
    println!("🔧 Surface created");
    
    // Request adapter
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .ok_or("Failed to find adapter")?;
    
    println!("🔧 GPU Adapter: {}", adapter.get_info().name);
    println!("🔧 Backend: {:?}", adapter.get_info().backend);
    
    // Request device and queue
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await
        .map_err(|e| format!("Failed to create device: {}", e))?;
    
    println!("🔧 Device and queue created");
    
    // Configure surface
    let size = window.inner_size();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: surface.get_capabilities(&adapter).alpha_modes[0],
        view_formats: vec![],
    };
    
    surface.configure(&device, &config);
    println!("🔧 Surface configured");
    
    Ok(())
}