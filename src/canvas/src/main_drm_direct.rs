//! Canvas - Direct DRM Display Server without session management
//! For testing in environments without seatd/libseat

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
    utils::{DeviceFd, Rectangle, Scale},
};

use tracing::{error, info};

// Type alias for our compositor
type GbmDrmCompositor = DrmCompositor<
    GbmAllocator<DrmDeviceFd>,
    GbmFramebufferExporter<DrmDeviceFd>,
    Option<OutputPresentationFeedback>,
    DrmDeviceFd,
>;

// Supported formats
const SUPPORTED_FORMATS: &[Fourcc] = &[
    Fourcc::Abgr8888,
    Fourcc::Argb8888,
];

// Clear color - blue background
const CLEAR_COLOR: Color32F = Color32F::new(0.1, 0.1, 0.2, 1.0);

// Simple test element
#[derive(Debug, Clone)]
struct TestElement {
    id: Id,
    commit: CommitCounter,
}

impl TestElement {
    fn new() -> Self {
        Self {
            id: Id::new(),
            commit: CommitCounter::default(),
        }
    }
}

impl Element for TestElement {
    fn id(&self) -> &Id {
        &self.id
    }

    fn current_commit(&self) -> CommitCounter {
        self.commit
    }

    fn src(&self) -> Rectangle<f64, smithay::utils::Buffer> {
        Rectangle::from_size((100.0, 100.0).into())
    }

    fn geometry(&self, _scale: Scale<f64>) -> Rectangle<i32, smithay::utils::Physical> {
        Rectangle::from_size((100, 100).into())
    }
}

impl RenderElement<GlesRenderer> for TestElement {
    fn draw(
        &self,
        _frame: &mut <GlesRenderer as smithay::backend::renderer::RendererSuper>::Frame<'_, '_>,
        _src: Rectangle<f64, smithay::utils::Buffer>,
        _dst: Rectangle<i32, smithay::utils::Physical>,
        _damage: &[Rectangle<i32, smithay::utils::Physical>],
        _opaque_regions: &[Rectangle<i32, smithay::utils::Physical>],
    ) -> Result<(), <GlesRenderer as smithay::backend::renderer::RendererSuper>::Error> {
        Ok(())
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
}

impl CanvasState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Open DRM device directly
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

        info!("DRM device opened");

        // Create DRM device
        // Set disable_connectors to false to avoid needing master for initialization
        let (drm, _notifier) = DrmDevice::new(drm_fd.clone(), false)?;
        info!("DRM device created (non-master mode)");

        // Create GBM device
        let gbm = GbmDevice::new(drm_fd.clone())?;
        info!("GBM device created");

        // Create EGL display and context
        let egl_display = unsafe { EGLDisplay::new(gbm.clone())? };
        let egl_context = EGLContext::new(&egl_display)?;

        // Create renderer
        let renderer = unsafe { GlesRenderer::new(egl_context)? };
        info!("GLES renderer created");

        Ok(Self {
            drm_fd,
            drm,
            gbm,
            renderer,
            surfaces: HashMap::new(),
        })
    }

    fn scan_connectors(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let res = self.drm_fd.resource_handles()?;
        info!("Found {} connectors", res.connectors().len());

        for connector_handle in res.connectors() {
            let connector = self.drm_fd.get_connector(*connector_handle, false)?;

            if connector.state() == connector::State::Connected {
                info!("Connected output: {:?}", connector.interface());

                // Find an available CRTC
                for crtc in res.crtcs() {
                    if self.surfaces.contains_key(crtc) {
                        continue;
                    }

                    // Get the preferred mode
                    let mode = connector.modes()
                        .iter()
                        .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
                        .or_else(|| connector.modes().first())
                        .ok_or("No modes available")?;

                    info!("Using mode: {}x{} @ {}Hz",
                        mode.size().0, mode.size().1, mode.vrefresh());

                    // Create surface
                    self.create_surface(*crtc, connector.clone(), mode.clone())?;
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

        // Create Smithay output
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
                model: "DRM".to_string(),
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

        let compositor = DrmCompositor::new(
            &output,
            surface,
            None,
            allocator,
            exporter,
            SUPPORTED_FORMATS.iter().copied(),
            renderer_formats,
            self.drm.cursor_size(),
            Some(self.gbm.clone()),
        )?;

        self.surfaces.insert(crtc, SurfaceData {
            compositor,
            output,
        });

        info!("Surface created for CRTC {:?}", crtc);
        Ok(())
    }

    fn render(&mut self) {
        let crtcs: Vec<_> = self.surfaces.keys().cloned().collect();

        for crtc in crtcs {
            if let Some(surface) = self.surfaces.get_mut(&crtc) {
                let elements = vec![TestElement::new()];

                match surface.compositor.render_frame(
                    &mut self.renderer,
                    &elements,
                    CLEAR_COLOR,
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("Canvas Direct DRM - No Session Management");
    info!("==========================================");

    // Create state
    let mut state = CanvasState::new()?;

    // Scan for connected outputs
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

    info!("Canvas Direct DRM ready");
    info!("Rendering blue background at 60 FPS");
    info!("Press Ctrl+C to exit");

    // Main loop
    loop {
        event_loop.dispatch(Some(std::time::Duration::from_millis(16)), &mut state)?;
    }
}