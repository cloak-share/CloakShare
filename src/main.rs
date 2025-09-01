mod pixel_conversion;
mod gpu_renderer;
mod screen_capture;

use crate::{gpu_renderer::GpuRenderer, screen_capture::ScreenCaptureManager};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

/// SafeMirror: Simplified structure that coordinates GPU rendering and screen capture
struct SafeMirror {
    gpu_renderer: GpuRenderer,
    screen_capture: ScreenCaptureManager,
}

impl SafeMirror {
    async fn new(window: Arc<Window>) -> Self {
        let gpu_renderer = GpuRenderer::new(window).await;
        let mut screen_capture = ScreenCaptureManager::new();
        
        if let Err(e) = screen_capture.start_capture() {
            eprintln!("Failed to start screen capture: {}", e);
        }
        
        Self {
            gpu_renderer,
            screen_capture,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.gpu_renderer.resize(new_size);
    }

    fn update_and_render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Get latest frame or use test pattern
        let texture_data = self.screen_capture.get_latest_frame()
            .unwrap_or_else(|| self.gpu_renderer.create_test_pattern());
        
        // Update GPU texture and render
        self.gpu_renderer.update_texture(&texture_data);
        self.gpu_renderer.render()
    }
}

/// Application state for winit event loop
struct App {
    safe_mirror: Option<SafeMirror>,
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("CloakShare - Safe Mirror")
                        .with_inner_size(winit::dpi::LogicalSize::new(1280, 720)),
                )
                .unwrap(),
        );
        
        self.window = Some(window.clone());
        self.safe_mirror = Some(pollster::block_on(SafeMirror::new(window)));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let Some(safe_mirror) = &mut self.safe_mirror {
            match event {
                WindowEvent::CloseRequested => {
                    println!("Close button was pressed!");
                    event_loop.exit();
                }
                WindowEvent::Resized(physical_size) => {
                    safe_mirror.resize(physical_size);
                }
                WindowEvent::RedrawRequested => {
                    match safe_mirror.update_and_render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            if let Some(window) = &self.window {
                                safe_mirror.resize(window.inner_size());
                            }
                        }
                        Err(wgpu::SurfaceError::OutOfMemory) => {
                            event_loop.exit();
                        }
                        Err(e) => eprintln!("Render error: {e:?}"),
                    }
                }
                _ => {}
            }
        }

        // Request continuous redraws
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() {
    println!("Starting CloakShare Safe Mirror...");

    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        safe_mirror: None,
        window: None,
    };

    event_loop.run_app(&mut app).unwrap();
}