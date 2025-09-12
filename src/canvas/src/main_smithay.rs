//! Canvas - Minimal Smithay Compositor for Conversational Computer
//! Based on Smithay minimal example

use std::{os::unix::io::OwnedFd, sync::Arc};

use smithay::{
    backend::{
        input::{InputEvent, KeyboardKeyEvent},
        renderer::{
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                Kind,
            },
            gles::GlesRenderer,
            utils::{draw_render_elements, on_commit_buffer_handler},
            Color32F, Frame, Renderer,
        },
        winit::{self, WinitEvent},
    },
    delegate_compositor, delegate_data_device, delegate_seat, delegate_shm, delegate_xdg_shell,
    input::{keyboard::FilterResult, Seat, SeatHandler, SeatState},
    reexports::wayland_server::{protocol::wl_seat, Display},
    utils::{Rectangle, Serial, Transform},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            CompositorClientState, CompositorHandler, CompositorState,
        },
        selection::{
            data_device::{ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler},
            SelectionHandler,
        },
        shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState},
        shm::{ShmHandler, ShmState},
    },
};

use wayland_protocols::xdg::shell::server::xdg_toplevel;
use wayland_server::{
    backend::{ClientData, ClientId, DisconnectReason},
    protocol::{
        wl_buffer,
        wl_surface::{self, WlSurface},
    },
    Client, ListeningSocket,
};

mod font;
mod spoc_client;

// Conversation block types for our Canvas
#[derive(Clone, Debug)]
enum BlockRole {
    User,
    Assistant, 
    System,
    Terminal,
}

#[derive(Clone, Debug)]
struct Block {
    text: String,
    role: BlockRole,
    timestamp: std::time::SystemTime,
    height: usize,
    command: Option<String>,
}

// Our Canvas compositor state
struct Canvas {
    // Smithay required state
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    seat_state: SeatState<Self>,
    data_device_state: DataDeviceState,
    seat: Seat<Self>,
    
    // Canvas-specific state
    blocks: Vec<Block>,
    input_text: String,
    cursor_position: usize,
    scroll_offset: usize,
    spoc_available: bool,
}

// Client state
#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

// Implement required Smithay handlers
impl BufferHandler for Canvas {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl CompositorHandler for Canvas {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
    }
}

impl ShmHandler for Canvas {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl XdgShellHandler for Canvas {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // We don't want traditional windows in our conversational computer
        // But we need to handle this for compatibility
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}
    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}
    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {}
}

impl SelectionHandler for Canvas {
    type SelectionUserData = ();
}

impl DataDeviceHandler for Canvas {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for Canvas {}
impl ServerDndGrabHandler for Canvas {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}
}

impl SeatHandler for Canvas {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: smithay::input::pointer::CursorImageStatus) {}
}

// Delegate macros for Smithay
delegate_compositor!(Canvas);
delegate_shm!(Canvas);
delegate_xdg_shell!(Canvas);
delegate_seat!(Canvas);
delegate_data_device!(Canvas);

impl Canvas {
    fn new(display: &Display<Canvas>) -> Self {
        let dh = display.handle();
        
        let compositor_state = CompositorState::new::<Canvas>(&dh);
        let xdg_shell_state = XdgShellState::new::<Canvas>(&dh);
        let shm_state = ShmState::new::<Canvas>(&dh, vec![]);
        let mut seat_state = SeatState::new();
        let seat = seat_state.new_wl_seat(&dh, "canvas-seat");
        let data_device_state = DataDeviceState::new::<Canvas>(&dh);
        
        // Check if SPOC is available
        let spoc_available = spoc_client::SPOCClient::is_available();
        if spoc_available {
            println!("SPOC conductor is available");
        } else {
            println!("SPOC conductor not available - local mode");
        }
        
        // Initial welcome block
        let blocks = vec![Block {
            text: "Canvas - Smithay Conversational Computer".to_string(),
            role: BlockRole::System,
            timestamp: std::time::SystemTime::now(),
            height: 20,
            command: None,
        }];
        
        Self {
            compositor_state,
            xdg_shell_state,
            shm_state,
            seat_state,
            data_device_state,
            seat,
            blocks,
            input_text: String::new(),
            cursor_position: 0,
            scroll_offset: 0,
            spoc_available,
        }
    }
    
    fn handle_keyboard_input(&mut self, key_code: u32, pressed: bool) {
        if !pressed {
            return;
        }
        
        // Basic key handling - will expand with proper keysym mapping
        match key_code {
            28 => { // Enter key
                if !self.input_text.is_empty() {
                    self.send_message();
                }
            }
            14 => { // Backspace
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    self.input_text.remove(self.cursor_position);
                }
            }
            _ => {
                // For now, just print the key code
                println!("Key pressed: {}", key_code);
            }
        }
    }
    
    fn send_message(&mut self) {
        let message = self.input_text.clone();
        
        // Add user block
        self.blocks.push(Block {
            text: message.clone(),
            role: BlockRole::User,
            timestamp: std::time::SystemTime::now(),
            height: 30,
            command: None,
        });
        
        // Send to SPOC if available
        if self.spoc_available {
            match spoc_client::SPOCClient::query(&message) {
                Ok(response_blocks) => {
                    for block in response_blocks {
                        let role = match block.role.as_str() {
                            "assistant" => BlockRole::Assistant,
                            "system" => BlockRole::System,
                            "terminal" => BlockRole::Terminal,
                            _ => BlockRole::System,
                        };
                        
                        let text = block.content.or(block.output).unwrap_or_default();
                        
                        self.blocks.push(Block {
                            text,
                            role,
                            timestamp: std::time::SystemTime::now(),
                            height: 30,
                            command: block.command,
                        });
                    }
                }
                Err(e) => {
                    self.blocks.push(Block {
                        text: format!("Error: {}", e),
                        role: BlockRole::System,
                        timestamp: std::time::SystemTime::now(),
                        height: 20,
                        command: None,
                    });
                }
            }
        }
        
        // Clear input
        self.input_text.clear();
        self.cursor_position = 0;
    }
}

// Helper function for sending frame callbacks
fn send_frames_surface_tree(surface: &WlSurface, time: u32) {
    // Simplified for now - will implement proper frame callbacks later
    _ = (surface, time);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }
    
    println!("Canvas Smithay Compositor Starting...");
    
    // Create Wayland display
    let mut display: Display<Canvas> = Display::new()?;
    
    // Create Canvas state
    let mut state = Canvas::new(&display);
    
    // Create listening socket
    let listener = ListeningSocket::bind("wayland-canvas")?;
    std::env::set_var("WAYLAND_DISPLAY", "wayland-canvas");
    
    // Initialize winit backend for testing
    let (mut backend, mut winit) = winit::init::<GlesRenderer>()?;
    
    // Add keyboard to seat
    let keyboard = state.seat.add_keyboard(Default::default(), 200, 25)?;
    
    let start_time = std::time::Instant::now();
    
    println!("Canvas compositor ready on WAYLAND_DISPLAY=wayland-canvas");
    
    // Main event loop
    loop {
        // Handle winit events
        let status = winit.dispatch_new_events(|event| match event {
            WinitEvent::Resized { .. } => {}
            WinitEvent::Input(event) => match event {
                InputEvent::Keyboard { event } => {
                    // Handle keyboard input
                    let _result = keyboard.input::<(), _>(
                        &mut state,
                        event.key_code(),
                        event.state(),
                        0.into(),
                        0,
                        |state, _, _| {
                            let pressed = match event.state() {
                                smithay::backend::input::KeyState::Pressed => true,
                                smithay::backend::input::KeyState::Released => false,
                            };
                            state.handle_keyboard_input(event.key_code().into(), pressed);
                            FilterResult::Forward
                        },
                    );
                }
                _ => {}
            },
            _ => (),
        });
        
        // Check if we should exit using winit's re-exported PumpStatus
        use smithay::reexports::winit::platform::pump_events::PumpStatus;
        match status {
            PumpStatus::Exit(_) => break,
            _ => {}
        }
        
        // Render
        let size = backend.window_size();
        let damage = Rectangle::from_size(size);
        
        backend.bind().map(|(renderer, mut framebuffer)| {
            // Clear to dark background
            let mut frame = renderer
                .render(&mut framebuffer, size, Transform::Flipped180)
                .unwrap();
            frame.clear(Color32F::new(0.05, 0.05, 0.1, 1.0), &[damage]).unwrap();
            
            // TODO: Render conversation blocks here
            
            frame.finish().unwrap();
        });
        
        // Accept new clients
        if let Ok(Some(stream)) = listener.accept() {
            println!("New client connected");
            display.handle().insert_client(stream, Arc::new(ClientState::default()))?;
        }
        
        // Dispatch Wayland events
        display.dispatch_clients(&mut state)?;
        display.flush_clients()?;
    }
    
    Ok(())
}