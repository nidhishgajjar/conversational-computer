//! Canvas - DRM Display Server using Smithay
//! Direct hardware control. No apps. Just conversation.

use std::{
    collections::HashMap,
    path::Path,
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
            DrmDevice, DrmDeviceFd, DrmNode,
        },
        renderer::{
            gles::GlesRenderer,
            element::{Element, Id, RenderElement},
            utils::CommitCounter,
            Color32F, ImportDma, Renderer,
        },
        session::{
            libseat::LibSeatSession,
            Session,
        },
    },
    desktop::utils::OutputPresentationFeedback,
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{
        calloop::{self, EventLoop},
        drm::control::{connector, crtc, Device as ControlDevice, ModeTypeFlags},
        rustix::fs::OFlags,
    },
    utils::{DeviceFd, Rectangle, Scale},
};

use tracing::{error, info};

// Type alias for our compositor - using exact types from Smithay
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

// Clear color
const CLEAR_COLOR: Color32F = Color32F::new(0.1, 0.1, 0.2, 1.0);

// Simple render element for testing
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
        // For now, just clear - we'll add actual rendering later
        Ok(())
    }
}

struct SurfaceData {
    compositor: GbmDrmCompositor,
    output: Output,
}

struct BackendData {
    drm: DrmDevice,
    gbm: GbmDevice<DrmDeviceFd>,
    surfaces: HashMap<crtc::Handle, SurfaceData>,
    renderer: GlesRenderer,
}

struct CanvasState {
    session: LibSeatSession,
    backends: HashMap<DrmNode, BackendData>,
}

impl CanvasState {
    fn new(session: LibSeatSession) -> Self {
        Self {
            session,
            backends: HashMap::new(),
        }
    }
    
    fn device_added(&mut self, node: DrmNode, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        info!("Adding DRM device: {:?}", path);
        
        // Open device
        let fd = self.session.open(
            path,
            OFlags::RDWR | OFlags::CLOEXEC | OFlags::NOCTTY | OFlags::NONBLOCK,
        )?;
        
        let device_fd = DrmDeviceFd::new(DeviceFd::from(fd));
        
        // Create DRM device
        let (drm, _notifier) = DrmDevice::new(device_fd.clone(), true)?;
        
        // Create GBM device
        let gbm = GbmDevice::new(device_fd.clone())?;
        
        // Create renderer
        use smithay::backend::egl::{EGLContext, EGLDisplay};
        let renderer = unsafe {
            GlesRenderer::new(EGLContext::new(&EGLDisplay::new(gbm.clone())?)?)?
        };
        
        // Store backend
        self.backends.insert(
            node,
            BackendData {
                drm,
                gbm,
                surfaces: HashMap::new(),
                renderer,
            }
        );
        
        // Scan connectors
        self.device_changed(node);
        
        Ok(())
    }
    
    fn device_changed(&mut self, node: DrmNode) {
        // Collect connection info first to avoid borrow issues
        let mut connections = Vec::new();
        
        if let Some(backend) = self.backends.get_mut(&node) {
            // Get resources
            let res = match backend.drm.resource_handles() {
                Ok(res) => res,
                Err(e) => {
                    error!("Failed to get resources: {}", e);
                    return;
                }
            };
            
            // Check connectors
            for connector_handle in res.connectors() {
                if let Ok(connector) = backend.drm.get_connector(*connector_handle, false) {
                    if connector.state() == connector::State::Connected {
                        info!("Found connected output: {:?}", connector.interface());
                        
                        // Find CRTC
                        for crtc in res.crtcs() {
                            // Simple allocation - just use first available
                            if backend.surfaces.contains_key(crtc) {
                                continue;
                            }
                            
                            connections.push((connector, *crtc));
                            break;
                        }
                    }
                }
            }
        }
        
        // Now process connections without holding backend reference
        for (connector, crtc) in connections {
            self.connector_connected(node, connector, crtc);
        }
    }
    
    fn connector_connected(&mut self, node: DrmNode, connector: connector::Info, crtc: crtc::Handle) {
        let backend = match self.backends.get_mut(&node) {
            Some(b) => b,
            None => return,
        };
        
        // Get mode
        let mode = connector
            .modes()
            .iter()
            .find(|m| m.mode_type().contains(ModeTypeFlags::PREFERRED))
            .or_else(|| connector.modes().first())
            .copied();
        
        let mode = match mode {
            Some(m) => m,
            None => {
                error!("No modes available");
                return;
            }
        };
        
        // Create output
        let output_name = format!("{}-{}", connector.interface().as_str(), connector.interface_id());
        let (phys_w, phys_h) = connector.size().unwrap_or((0, 0));
        
        let output = Output::new(
            output_name.clone(),
            PhysicalProperties {
                size: (phys_w as i32, phys_h as i32).into(),
                subpixel: Subpixel::Unknown,
                make: "Canvas".to_string(),
                model: "DRM".to_string(),
            },
        );
        
        let mode_converted = Mode::from(mode);
        output.change_current_state(Some(mode_converted), None, None, Some((0, 0).into()));
        output.set_preferred(mode_converted);
        
        info!("Creating surface for {} ({}x{})", output_name, mode.size().0, mode.size().1);
        
        // Create DRM surface
        let surface = match backend.drm.create_surface(crtc, mode, &[connector.handle()]) {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to create surface: {}", e);
                return;
            }
        };
        
        // Create allocator for this surface
        let allocator = GbmAllocator::new(
            backend.gbm.clone(),
            GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
        );
        
        // Create framebuffer exporter
        let exporter = GbmFramebufferExporter::new(
            backend.gbm.clone(),
            None, // No cross-GPU import for now
        );
        
        // Get renderer formats
        let renderer_formats = backend.renderer.dmabuf_formats().clone();
        
        // Create compositor using the exact type signature
        let compositor = match DrmCompositor::new(
            &output,
            surface,
            None, // No specific planes
            allocator,
            exporter,
            SUPPORTED_FORMATS.iter().copied(),
            renderer_formats,
            backend.drm.cursor_size(),
            Some(backend.gbm.clone()),
        ) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to create compositor: {:?}", e);
                return;
            }
        };
        
        // Store surface
        backend.surfaces.insert(crtc, SurfaceData {
            compositor,
            output,
        });
        
        info!("Surface created for CRTC {:?}", crtc);
    }
    
    fn render(&mut self) {
        // Render to all surfaces
        for backend in self.backends.values_mut() {
            // Collect CRTCs to render (to avoid borrow issues)
            let crtcs: Vec<_> = backend.surfaces.keys().cloned().collect();
            
            for crtc in crtcs {
                if let Some(surface) = backend.surfaces.get_mut(&crtc) {
                    // Create test elements - in the future this will be conversation blocks
                    let elements = vec![TestElement::new()];
                    
                    // Render frame
                    match surface.compositor.render_frame(
                        &mut backend.renderer,
                        &elements,
                        CLEAR_COLOR,
                        FrameFlags::empty(),
                    ) {
                        Ok(result) => {
                            if !result.is_empty {
                                // Queue frame for presentation
                                if let Err(e) = surface.compositor.queue_frame(None) {
                                    error!("Failed to queue frame: {:?}", e);
                                    continue;
                                }
                                
                                // Mark frame as submitted
                                // In a real implementation, we'd wait for VBlank event
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();
    
    info!("Canvas DRM - Direct Display Control");
    info!("=====================================");
    
    // Create event loop
    let mut event_loop = EventLoop::try_new()?;
    let handle = event_loop.handle();
    
    // Initialize session
    let (session, _notifier) = LibSeatSession::new()?;
    info!("LibSeat session initialized");
    
    // Create state
    let mut state = CanvasState::new(session);
    
    // Add primary GPU
    let gpu = DrmNode::from_path("/dev/dri/card0")?;
    if let Some(path) = gpu.dev_path() {
        state.device_added(gpu, &path)?;
    }
    
    // Set up render timer (60 FPS)
    let _timer = handle.insert_source(
        calloop::timer::Timer::from_duration(std::time::Duration::from_millis(16)),
        |_, _, state: &mut CanvasState| {
            state.render();
            calloop::timer::TimeoutAction::ToDuration(std::time::Duration::from_millis(16))
        },
    )?;
    
    info!("Canvas DRM ready");
    info!("We own the display");
    
    // Main loop
    loop {
        event_loop.dispatch(Some(std::time::Duration::from_millis(16)), &mut state)?;
    }
}