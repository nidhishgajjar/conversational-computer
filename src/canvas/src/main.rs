//! Canvas - Conversational Computer Compositor
//! A minimal Wayland compositor designed for AI-first interaction

use anyhow::Result;
use smithay::{
    backend::{
        allocator::dmabuf::Dmabuf,
        renderer::{
            element::surface::WaylandSurfaceRenderElement,
            glow::GlowRenderer,
        },
        winit::{self, WinitEvent},
    },
    desktop::{Space, Window},
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{EventLoop, LoopHandle},
        wayland_server::{
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle,
        },
    },
    utils::{Rectangle, Size, Transform},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        dmabuf::DmabufState,
        output::OutputManagerState,
        seat::{Seat, SeatState},
        shell::xdg::{XdgShellState, XdgToplevelSurfaceData},
        shm::ShmState,
    },
};
use std::sync::Arc;
use std::time::Duration;

/// Canvas compositor state
pub struct Canvas {
    /// Wayland display
    display: Display<Canvas>,
    
    /// Event loop handle
    loop_handle: LoopHandle<'static, Canvas>,
    
    /// Compositor space (manages windows/surfaces)
    space: Space<Window>,
    
    /// Wayland protocol states
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    output_manager_state: OutputManagerState,
    seat_state: SeatState<Canvas>,
    
    /// Main seat
    seat: Seat<Canvas>,
    
    /// Background color (dark blue-gray)
    bg_color: [f32; 4],
}

impl Canvas {
    pub fn new(
        display: Display<Canvas>,
        loop_handle: LoopHandle<'static, Canvas>,
    ) -> Result<Self> {
        let dh = display.handle();
        
        // Create compositor state
        let compositor_state = CompositorState::new::<Canvas>(&dh);
        let xdg_shell_state = XdgShellState::new::<Canvas>(&dh);
        let shm_state = ShmState::new::<Canvas>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Canvas>(&dh);
        let mut seat_state = SeatState::new();
        
        // Create main seat
        let seat = seat_state.new_wl_seat(&dh, "seat0");
        
        Ok(Canvas {
            display,
            loop_handle,
            space: Space::default(),
            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            seat,
            bg_color: [0.08, 0.10, 0.14, 1.0], // Dark blue-gray
        })
    }
    
    /// Create an output (display)
    pub fn create_output(&mut self, name: String, size: Size<i32, smithay::utils::Physical>) {
        let mode = Mode {
            size,
            refresh: 60_000, // 60 Hz
        };
        
        let physical = PhysicalProperties {
            size: (size.w, size.h).into(),
            subpixel: Subpixel::Unknown,
            make: "Canvas".into(),
            model: "Virtual".into(),
        };
        
        let output = Output::new(
            name,
            physical,
        );
        
        output.create_global::<Canvas>(&self.display.handle());
        output.change_current_state(
            Some(mode),
            Some(Transform::Normal),
            None,
            Some((0, 0).into()),
        );
        output.set_preferred(mode);
        
        self.space.map_output(&output, (0, 0));
    }
    
    /// Render a frame
    pub fn render_frame(&mut self, renderer: &mut GlowRenderer, size: Size<i32, smithay::utils::Physical>) {
        // Clear to background color
        renderer.clear(self.bg_color);
        
        // TODO: Render SPOC interface elements here
        self.render_spoc_ui(renderer, size);
        
        // Render windows (if any - we won't have traditional windows)
        let elements = self.space
            .elements()
            .filter_map(|window| {
                self.space
                    .outputs()
                    .find(|o| {
                        let geometry = self.space.element_geometry(window)?;
                        o.geometry().overlaps_with(&geometry)
                    })
                    .map(|_| window.clone())
            })
            .flat_map(|window| {
                window.render_elements::<WaylandSurfaceRenderElement<GlowRenderer>>(
                    renderer,
                    (0, 0).into(),
                    1.0,
                    1.0,
                )
            });
        
        // Draw elements
        for element in elements {
            renderer.render_element(&element, size, Transform::Normal, 1.0).ok();
        }
    }
    
    /// Render SPOC UI elements
    fn render_spoc_ui(&self, _renderer: &mut GlowRenderer, _size: Size<i32, smithay::utils::Physical>) {
        // TODO: Implement SPOC input bar rendering
        // This will include:
        // - Input bar at bottom
        // - Conversation blocks
        // - Text rendering
    }
}

fn main() -> Result<()> {
    // Setup logging
    let logger = slog::Logger::root(
        slog_term::FullFormat::new(slog_term::TermDecorator::new().build()).build().fuse(),
        slog::o!(),
    );
    let _guard = slog_scope::set_global_logger(logger);
    
    println!("Canvas Compositor starting...");
    println!("Conversational Computer - AI-first interaction");
    
    // Create event loop
    let mut event_loop = EventLoop::try_new()?;
    let loop_handle = event_loop.handle();
    
    // Create Wayland display
    let display = Display::new()?;
    
    // Create Canvas compositor
    let mut canvas = Canvas::new(display.clone(), loop_handle.clone())?;
    
    // For testing, use winit backend (window)
    // In production, this would be DRM/TTY backend
    let (backend, winit_event) = winit::init()?;
    
    // Create output
    canvas.create_output("canvas-1".into(), (1920, 1080).into());
    
    // Insert winit event source
    event_loop
        .handle()
        .insert_source(winit_event, |event, _, canvas| {
            match event {
                WinitEvent::Resized { size, .. } => {
                    // Handle resize
                    println!("Window resized to {:?}", size);
                }
                WinitEvent::Input(input) => {
                    // Handle input
                    println!("Input event: {:?}", input);
                    
                    // TODO: Process input for SPOC
                }
                WinitEvent::Redraw => {
                    // Render frame
                    // canvas.render_frame(...);
                }
                WinitEvent::CloseRequested => {
                    // Exit
                    std::process::exit(0);
                }
                _ => {}
            }
        })?;
    
    // Setup socket
    let socket_name = display
        .add_socket_auto()
        .expect("Failed to create socket");
    
    println!("Canvas running on WAYLAND_DISPLAY={}", socket_name.to_string_lossy());
    println!("Press Ctrl+C to exit");
    
    // Run event loop
    loop {
        event_loop.dispatch(Duration::from_millis(16), &mut canvas)?;
        display.flush_clients()?;
    }
}

// Smithay delegate implementations
delegate_compositor!(Canvas);
delegate_xdg_shell!(Canvas);
delegate_shm!(Canvas);
delegate_output!(Canvas);
delegate_seat!(Canvas);

// Macro implementations
use smithay::delegate_compositor;
use smithay::delegate_xdg_shell;
use smithay::delegate_shm;
use smithay::delegate_output;
use smithay::delegate_seat;