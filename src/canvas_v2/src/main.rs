use anyhow::Result;
use std::num::NonZeroU32;
use std::rc::Rc;
use tracing::info;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent, ElementState, KeyEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowBuilder;

mod spoc_input;
use spoc_input::SpocInput;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();
    
    info!("Starting Canvas - Conversational Computer");
    info!("Foundation for SPOC-driven interface");
    
    // Create event loop and window
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Canvas - Conversational Computer")
        .with_inner_size(LogicalSize::new(1280.0, 720.0))
        .build(&event_loop)?;
    
    let window = Rc::new(window);
    
    // Create software buffer for rendering
    let context = softbuffer::Context::new(&window).unwrap();
    let mut surface = softbuffer::Surface::new(&context, &window).unwrap();
    
    info!("Canvas ready");
    info!("Building foundation for conversation-first computing");
    
    // Create SPOC input with Spotlight aesthetics
    let mut spoc_input = SpocInput::new();
    
    // Main event loop
    let window_clone = window.clone();
    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    info!("Shutting down Canvas");
                    elwt.exit();
                }
                WindowEvent::Resized(size) => {
                    info!("Window resized: {}x{}", size.width, size.height);
                    if size.width > 0 && size.height > 0 {
                        surface.resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        ).unwrap();
                    }
                }
                WindowEvent::RedrawRequested => {
                    // Get window size
                    let size = window_clone.inner_size();
                    let width = size.width as usize;
                    let height = size.height as usize;
                    
                    // Ensure surface is sized properly
                    if width > 0 && height > 0 {
                        surface.resize(
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        ).unwrap();
                    }
                    
                    // Render frame
                    let mut buffer = surface.buffer_mut().unwrap();
                    
                    // Skip frame if buffer is invalid
                    if buffer.len() > 0 && width > 0 && height > 0 {
                    
                    // Clear to dark Canvas background
                    for y in 0..height {
                        for x in 0..width {
                            let index = y * width + x;
                            if index < buffer.len() {
                                // Dark background - Canvas signature
                                buffer[index] = 0xFF0A0A0F;
                            }
                        }
                    }
                    
                    // Draw Canvas foundation text
                    draw_canvas_text(&mut buffer, width, height);
                    
                    // Draw SPOC input with Spotlight aesthetics
                    spoc_input.render(&mut buffer, width, height);
                    
                    } // end if buffer valid
                    
                    // Present the frame
                    buffer.present().unwrap();
                }
                WindowEvent::KeyboardInput { event: KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state,
                    text,
                    ..
                }, .. } => {
                    if state == ElementState::Pressed {
                        match keycode {
                            KeyCode::Escape => {
                                spoc_input.clear();
                            }
                            KeyCode::Enter => {
                                let text = spoc_input.get_text();
                                if !text.is_empty() {
                                    info!("SPOC Input: {}", text);
                                    spoc_input.clear();
                                }
                            }
                            KeyCode::Backspace => {
                                spoc_input.backspace();
                            }
                            KeyCode::ArrowUp => {
                                spoc_input.move_up();
                            }
                            KeyCode::ArrowDown => {
                                spoc_input.move_down();
                            }
                            KeyCode::ArrowLeft => {
                                spoc_input.move_left();
                            }
                            KeyCode::ArrowRight => {
                                spoc_input.move_right();
                            }
                            _ => {
                                // Handle text input
                                if let Some(text) = text {
                                    for c in text.chars() {
                                        if !c.is_control() {
                                            spoc_input.add_char(c);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            Event::AboutToWait => {
                window_clone.request_redraw();
            }
            _ => {}
        }
    })?;
    
    Ok(())
}

fn draw_canvas_text(buffer: &mut [u32], width: usize, height: usize) {
    // Draw "CANVAS" centered
    let text = "CANVAS";
    let char_width = 10;
    let text_width = text.len() * char_width;
    
    let start_x = (width - text_width) / 2;
    let start_y = height / 2 - 20;
    
    // Simple text placeholder
    let color = 0xFF2A2A3A;
    for x in start_x..(start_x + text_width) {
        if start_y < height && x < width {
            buffer[start_y * width + x] = color;
        }
    }
    
    // Subtitle
    let subtitle = "Conversational Computer";
    let subtitle_width = subtitle.len() * 7;
    let subtitle_x = (width - subtitle_width) / 2;
    let subtitle_y = start_y + 30;
    
    if subtitle_y < height {
        for x in subtitle_x..(subtitle_x + subtitle_width) {
            if x < width {
                buffer[subtitle_y * width + x] = 0xFF1A1A2A;
            }
        }
    }
}