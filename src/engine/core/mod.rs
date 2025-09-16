use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

pub struct Engine {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    // Graphics renderer will be added here
    // Audio system will be added here
    // Input manager will be added here
}

impl Engine {
    pub async fn new() -> Self {
        env_logger::init();

        let event_loop = EventLoop::new().unwrap();
        let window = WindowBuilder::new()
            .with_title("Robin Game Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
            .build(&event_loop)
            .unwrap();

        Self {
            window,
            event_loop: Some(event_loop),
        }
    }

    pub fn run(mut self) {
        let event_loop = self.event_loop.take().unwrap();
        
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.window.id() => match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(physical_size) => {
                        log::info!("Window resized to {:?}", physical_size);
                    }
                    WindowEvent::RedrawRequested => {
                        // Render frame here
                        // TODO: Add actual rendering logic here
                        log::debug!("Redraw requested");
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    self.window.request_redraw();
                }
                _ => {}
            }
        }).unwrap();
    }
}