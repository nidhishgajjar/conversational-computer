mod canvas;
mod input_field;
mod message;
mod keybindings;
mod animation;

use canvas::Canvas;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create window (temporary - will be framebuffer later)
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Canvas - Conversational Computer")
        .with_inner_size(winit::dpi::LogicalSize::new(1280, 800))
        .with_decorations(false) // No window chrome
        .with_transparent(true)
        .build(&event_loop)?;

    // Initialize Canvas with Skia
    let mut canvas = Canvas::new(&window)?;

    // Run event loop
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                WindowEvent::RedrawRequested => {
                    canvas.render();
                    window.request_redraw();
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    canvas.handle_keyboard(event);
                    window.request_redraw();
                }
                WindowEvent::ModifiersChanged(modifiers) => {
                    canvas.update_modifiers(modifiers.state());
                }
                WindowEvent::Resized(size) => {
                    canvas.resize(size.width, size.height);
                }
                _ => {}
            }
            Event::AboutToWait => {
                // Update animations
                if canvas.update_animations() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}