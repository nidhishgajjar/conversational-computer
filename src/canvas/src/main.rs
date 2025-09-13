//! Canvas - SPOC Input Interface
//! macOS Spotlight-inspired conversational interface

use std::{
    collections::HashMap,
    fs::OpenOptions,
    os::unix::io::{IntoRawFd, OwnedFd, FromRawFd},
};

use smithay::{
    backend::{
        allocator::{
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Fourcc,
        },
        drm::{
            compositor::{DrmCompositor, FrameFlags},
            exporter::gbm::GbmFramebufferExporter,
            DrmDevice, DrmDeviceFd,
        },
        egl::{EGLDisplay, EGLContext},
        renderer::{
            gles::GlesRenderer,
            element::{Element, Id, RenderElement},
            utils::CommitCounter,
            Color32F, ImportDma,
        },
    },
    desktop::utils::OutputPresentationFeedback,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{self, EventLoop},
        drm::control::{self, connector, crtc, Device as ControlDevice, ModeTypeFlags},
    },
    utils::{DeviceFd, Rectangle, Scale, Point, Size},
};

use tracing::{error, info};

mod spoc_client;

// Type alias for our compositor
type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    Option<OutputPresentationFeedback>,
    DrmDeviceFd,
>;

// Colors
const BG_COLOR: Color32F = Color32F::new(0.05, 0.05, 0.1, 1.0); // Dark background
const INPUT_BG: Color32F = Color32F::new(0.15, 0.15, 0.2, 0.95); // Semi-transparent input bg
const TEXT_COLOR: Color32F = Color32F::new(0.9, 0.9, 0.95, 1.0); // Light text

// Dimensions (Spotlight-like)
const INPUT_WIDTH: i32 = 680;
const INPUT_HEIGHT: i32 = 56;
const INPUT_PADDING: i32 = 16;
const BORDER_RADIUS: i32 = 10;

// SPOC Input Field
#[derive(Debug)]
struct SPOCInput {
    id: Id,
    commit: CommitCounter,
    text: String,
    cursor_pos: usize,
    multiline: bool,
    position: Point<i32, smithay::utils::Physical>,
    size: Size<i32, smithay::utils::Physical>,
}

impl SPOCInput {
    fn new(screen_size: Size<i32, smithay::utils::Physical>) -> Self {
        // Center horizontally, position at bottom of screen
        let x = (screen_size.w - INPUT_WIDTH) / 2;
        let y = screen_size.h - INPUT_HEIGHT - 40; // 40px from bottom

        Self {
            id: Id::new(),
            commit: CommitCounter::default(),
            text: String::new(),
            cursor_pos: 0,
            multiline: false,
            position: Point::from((x, y)),
            size: Size::from((INPUT_WIDTH, INPUT_HEIGHT)),
        }
    }

    fn handle_key(&mut self, key: u32, pressed: bool) {
        if !pressed {
            return;
        }

        // Key codes (simplified - in real implementation use xkbcommon)
        match key {
            28 => { // Enter
                if self.multiline {
                    self.text.push('\n');
                    self.cursor_pos += 1;
                } else {
                    // Submit to SPOC
                    self.submit();
                }
            }
            14 => { // Backspace
                if self.cursor_pos > 0 {
                    self.text.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }
            56 => { // Alt/Option - toggle multiline
                self.multiline = !self.multiline;
                if self.multiline {
                    self.size.h = INPUT_HEIGHT * 3;
                } else {
                    self.size.h = INPUT_HEIGHT;
                }
            }
            _ => {
                // Add character (simplified)
                if let Some(ch) = keycode_to_char(key) {
                    self.text.insert(self.cursor_pos, ch);
                    self.cursor_pos += 1;
                }
            }
        }

        self.commit.increment();
    }

    fn submit(&mut self) {
        if !self.text.is_empty() {
            info!("SPOC Query: {}", self.text);
            // TODO: Send to SPOC
            self.text.clear();
            self.cursor_pos = 0;
        }
    }
}

impl Element for SPOCInput {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn src(&self) -> Rectangle<f64, smithay::utils::Buffer> {
        Rectangle::from_size((self.size.w as f64, self.size.h as f64).into())
    }

    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, smithay::utils::Physical> {
        Rectangle::new(self.position, self.size)
    }
}

impl RenderElement<GlesRenderer> for SPOCInput {
    fn draw(
        &self,
        _frame: &mut <GlesRenderer as smithay::backend::renderer::RendererSuper>::Frame<'_, '_>,
        _src: Rectangle<f64, smithay::utils::Buffer>,
        _dst: Rectangle<i32, smithay::utils::Physical>,
        _damage: &[Rectangle<i32, smithay::utils::Physical>],
        _opaque_regions: &[Rectangle<i32, smithay::utils::Physical>],
    ) -> Result<(), <GlesRenderer as smithay::backend::renderer::RendererSuper>::Error> {
        // For now, just return Ok - we'll render a solid color element instead
        Ok(())
    }
}

// Simple keycode to char mapping (replace with xkbcommon in production)
fn keycode_to_char(keycode: u32) -> Option<char> {
    match keycode {
        16 => Some('q'),
        17 => Some('w'),
        18 => Some('e'),
        19 => Some('r'),
        20 => Some('t'),
        21 => Some('y'),
        22 => Some('u'),
        23 => Some('i'),
        24 => Some('o'),
        25 => Some('p'),
        30 => Some('a'),
        31 => Some('s'),
        32 => Some('d'),
        33 => Some('f'),
        34 => Some('g'),
        35 => Some('h'),
        36 => Some('j'),
        37 => Some('k'),
        38 => Some('l'),
        44 => Some('z'),
        45 => Some('x'),
        46 => Some('c'),
        47 => Some('v'),
        48 => Some('b'),
        49 => Some('n'),
        50 => Some('m'),
        57 => Some(' '),
        _ => None,
    }
}

struct SurfaceData {
    compositor: GbmDrmCompositor,
    output: Output,
}

struct CanvasState {
    drm_fd: DrmDeviceFd,
    drm: DrmDevice,
    gbm: GbmDevice<DrmDeviceFd>,
    renderer: GlesRenderer,
    surfaces: HashMap<crtc::Handle, SurfaceData>,
    spoc_input: Option<SPOCInput>,
    screen_size: Size<i32, smithay::utils::Physical>,
}

impl CanvasState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Open DRM device
        let gpu_path = "/dev/dri/card0";
        info!("Opening DRM device: {}", gpu_path);

        let gpu_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(gpu_path)?;

        let raw_fd = gpu_file.into_raw_fd();
        let owned_fd = unsafe { OwnedFd::from_raw_fd(raw_fd) };
        let device_fd = DeviceFd::from(owned_fd);
        let drm_fd = DrmDeviceFd::new(device_fd);

        // Create DRM device
        let (drm, _notifier) = DrmDevice::new(drm_fd.clone(), false)?;
        info!("DRM device created");

        // Create GBM device
        let gbm = GbmDevice::new(drm_fd.clone())?;

        // Create EGL context and renderer
        let egl_display = unsafe { EGLDisplay::new(gbm.clone())? };
        let egl_context = EGLContext::new(&egl_display)?;
        let renderer = unsafe { GlesRenderer::new(egl_context)? };
        info!("Renderer created");

        Ok(Self {
            drm_fd,
            drm,
            gbm,
            renderer,
            surfaces: HashMap::new(),
            spoc_input: None,
            screen_size: Size::from((1280, 800)), // Default, will be updated
        })
    }

    fn scan_connectors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let res = self.drm_fd.resource_handles()?;

        for connector_handle in res.connectors() {
            let connector = self.drm_fd.get_connector(*connector_handle, false)?;

            if connector.state() == connector::State::Connected {
                info!("Connected output: {:?}", connector.interface());

                for crtc in res.crtcs() {
                    if self.surfaces.contains_key(crtc) {
                        continue;
                    }

                    let mode = connector.modes()
                        .iter()
                        .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
                        .or_else(|| connector.modes().first())
                        .ok_or("No modes available")?;

                    info!("Using mode: {}x{} @ {}Hz",
                        mode.size().0, mode.size().1, mode.vrefresh());

                    self.create_surface(*crtc, connector.clone(), mode.clone())?;

                    // Update screen size
                    self.screen_size = Size::from((mode.size().0 as i32, mode.size().1 as i32));

                    // Create SPOC input
                    self.spoc_input = Some(SPOCInput::new(self.screen_size));

                    break;
                }
            }
        }

        Ok(())
    }

    fn create_surface(
        &mut self,
        crtc: crtc::Handle,
        connector: connector::Info,
        mode: control::Mode,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create DRM surface
        let surface = self.drm.create_surface(
            crtc,
            mode.clone(),
            &[connector.handle()],
        )?;

        // Create output
        let (width, height) = mode.size();
        let refresh = (mode.vrefresh() as f32 * 1000.0) as i32;
        let smithay_mode = Mode {
            size: (width as i32, height as i32).into(),
            refresh,
        };

        let output = Output::new(
            format!("{:?}", connector.interface()),
            PhysicalProperties {
                size: (width as i32, height as i32).into(),
                subpixel: Subpixel::Unknown,
                make: "Canvas".to_string(),
                model: "SPOC".to_string(),
            },
        );

        output.change_current_state(
            Some(smithay_mode),
            None,
            None,
            Some((0, 0).into()),
        );
        output.set_preferred(smithay_mode);

        // Create allocator and compositor
        let allocator = GbmAllocator::new(
            self.gbm.clone(),
            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
        );

        let exporter = GbmFramebufferExporter::new(
            self.gbm.clone(),
            None,
        );

        let renderer_formats = self.renderer.dmabuf_formats().clone();
        let supported_formats = &[Fourcc::Abgr8888, Fourcc::Argb8888];

        let compositor = DrmCompositor::new(
            &output,
            surface,
            None,
            allocator,
            exporter,
            supported_formats.iter().copied(),
            renderer_formats,
            self.drm.cursor_size(),
            Some(self.gbm.clone()),
        )?;

        self.surfaces.insert(crtc, SurfaceData {
            compositor,
            output,
        });

        info!("Surface created");
        Ok(())
    }

    fn render(&mut self) {
        let crtcs: Vec<_> = self.surfaces.keys().cloned().collect();

        for crtc in crtcs {
            if let Some(surface) = self.surfaces.get_mut(&crtc) {
                // For now, just render background color, no elements
                let elements: Vec<SPOCInput> = Vec::new();

                match surface.compositor.render_frame(
                    &mut self.renderer,
                    &elements,
                    BG_COLOR, // This should fill the whole screen with dark blue
                    FrameFlags::empty(),
                ) {
                    Ok(result) => {
                        if !result.is_empty {
                            if let Err(e) = surface.compositor.queue_frame(None) {
                                error!("Failed to queue frame: {:?}", e);
                                continue;
                            }

                            if let Err(e) = surface.compositor.frame_submitted() {
                                error!("Failed to mark frame as submitted: {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to render frame: {:?}", e);
                    }
                }
            }
        }
    }

}

// Make SPOCInput cloneable for rendering
impl Clone for SPOCInput {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            commit: self.commit,
            text: self.text.clone(),
            cursor_pos: self.cursor_pos,
            multiline: self.multiline,
            position: self.position,
            size: self.size,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("Canvas SPOC Interface");
    info!("====================");

    // Create state
    let mut state = CanvasState::new()?;

    // Scan for outputs
    state.scan_connectors()?;

    if state.surfaces.is_empty() {
        error!("No connected outputs found!");
        return Err("No displays connected".into());
    }

    // Create event loop
    let mut event_loop = EventLoop::try_new()?;
    let handle = event_loop.handle();

    // Set up render timer (60 FPS)
    let _timer = handle.insert_source(
        calloop::timer::Timer::from_duration(std::time::Duration::from_millis(16)),
        |_, _, state: &mut CanvasState| {
            state.render();
            calloop::timer::TimeoutAction::ToDuration(std::time::Duration::from_millis(16))
        },
    )?;

    info!("SPOC Interface ready");
    info!("Press keys to type, Option for multiline, Enter to submit");

    // Main loop
    loop {
        event_loop.dispatch(Some(std::time::Duration::from_millis(16)), &mut state)?;
    }
}