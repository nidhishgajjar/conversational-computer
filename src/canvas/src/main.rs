//! Canvas - Minimal DRM Display Control
//! Direct framebuffer rendering without compositor abstractions

use std::{
    fs::OpenOptions,
    os::unix::io::{IntoRawFd, OwnedFd, FromRawFd},
};

use smithay::{
    backend::{
        allocator::{
            dmabuf::{AsDmabuf, Dmabuf},
            gbm::{GbmAllocator, GbmBufferFlags, GbmDevice},
            Allocator, Fourcc,
        },
        drm::{DrmDevice, DrmDeviceFd},
        egl::{EGLDisplay, EGLContext},
        renderer::{
            gles::GlesRenderer,
            Bind, Frame, Renderer,
        },
    },
    reexports::drm::{
        control::{connector, Device as ControlDevice},
    },
    utils::{DeviceFd, Size, Transform},
};

use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    info!("Canvas - DRM Display Control");
    info!("============================");

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

    info!("✓ DRM device opened");

    // Create DRM device - this manages the display hardware
    let (mut drm, _notifier) = DrmDevice::new(drm_fd.clone(), false)?;
    info!("✓ DRM device created");

    // Create GBM device - Generic Buffer Management for GPU memory
    let gbm = GbmDevice::new(drm_fd.clone())?;
    info!("✓ GBM device created");

    // Create EGL context and renderer for OpenGL ES rendering
    let egl_display = unsafe { EGLDisplay::new(gbm.clone())? };
    let egl_context = EGLContext::new(&egl_display)?;
    let mut renderer = unsafe { GlesRenderer::new(egl_context)? };
    info!("✓ GLES renderer created");

    // Create GBM allocator for managing GPU buffers
    // RENDERING flag: buffer can be rendered to
    // SCANOUT flag: buffer can be displayed on screen
    let allocator = GbmAllocator::new(
        gbm.clone(),
        GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
    );
    info!("✓ GBM allocator created");

    // Display detection and setup
    let res = drm_fd.resource_handles()?;
    info!("Found {} connectors", res.connectors().len());

    let mut surface = None;
    let mut mode = None;

    for connector_handle in res.connectors() {
        let connector = drm_fd.get_connector(*connector_handle, false)?;

        if connector.state() == connector::State::Connected {
            info!("✓ Connected display: {:?}", connector.interface());

            // Get the preferred display mode (usually native resolution)
            // Falls back to first available mode if no preferred mode exists
            if let Some(m) = connector.modes().first() {
                info!("  Mode: {}x{} @ {}Hz", m.size().0, m.size().1, m.vrefresh());
                mode = Some(m.clone());

                // Find a suitable CRTC (CRT Controller - manages display output)
                // Each connector needs an encoder which connects to a CRTC
                for encoder_handle in connector.encoders() {
                    if let Ok(encoder) = drm_fd.get_encoder(*encoder_handle) {
                        // Use current CRTC if encoder already has one, otherwise find first available
                        let crtc_handle = encoder.crtc().unwrap_or_else(|| {
                            res.crtcs().iter().copied().next().unwrap()
                        });

                        // Create DRM surface - this connects a CRTC to connectors
                        // The surface manages the display pipeline for this screen
                        match drm.create_surface(
                            crtc_handle,
                            m.clone(),
                            &[*connector_handle],
                        ) {
                            Ok(s) => {
                                surface = Some((s, *connector_handle));
                                info!("✓ DRM surface created on CRTC {:?}", crtc_handle);
                                break;
                            }
                            Err(e) => {
                                info!("  Failed on CRTC {:?}: {}", crtc_handle, e);
                            }
                        }
                    }
                }

                if surface.is_some() {
                    break;
                }
            }
        }
    }

    let (mut drm_surface, _connector) = surface
        .ok_or_else(|| "No suitable display found")?;
    let display_mode = mode.ok_or_else(|| "No display mode found")?;

    info!("");
    info!("Starting minimal render loop - clearing with solid color");
    info!("Press Ctrl+C to exit");

    // Simple render loop - creates frames and displays them
    loop {
        // Create a GPU buffer for this frame
        // Size matches display resolution, format is XRGB (no alpha)
        let size = Size::from((display_mode.size().0 as i32, display_mode.size().1 as i32));
        let buffer = allocator
            .create_buffer(size.w as u32, size.h as u32, Fourcc::Xrgb8888, &[])
            .map_err(|e| format!("Failed to create buffer: {:?}", e))?;

        // Export the GbmBuffer as a Dmabuf for rendering
        // Dmabuf is a cross-device buffer sharing mechanism
        let mut dmabuf: Dmabuf = buffer.export()
            .map_err(|e| format!("Failed to export buffer: {:?}", e))?;

        // Bind the dmabuf as render target for the renderer
        let mut target = renderer.bind(&mut dmabuf)
            .map_err(|e| format!("Failed to bind dmabuf: {:?}", e))?;

        // Start a new frame for rendering operations
        // The render method needs a mutable reference to the target
        {
            let mut frame = renderer.render(&mut target, size, Transform::Normal)
                .map_err(|e| format!("Failed to start frame: {:?}", e))?;

            // Clear the entire frame with dark gray color (R=0.15, G=0.15, B=0.15)
            // Color needs to be converted to Color32F type
            frame.clear([0.15, 0.15, 0.15, 1.0].into(), &[]);

            // Finish the frame and submit rendering commands
            frame.finish()
                .map_err(|e| format!("Failed to finish frame: {:?}", e))?;
        } // Frame is dropped here, releasing the render target

        // Get the primary plane for this CRTC
        // Planes are hardware layers that can display buffers
        let planes = drm.planes(&drm_surface.crtc())
            .map_err(|e| format!("Failed to get planes: {:?}", e))?;

        // Get the first primary plane handle
        let primary_plane = planes.primary.iter().next()
            .ok_or_else(|| "No primary plane found")?;

        // Claim the plane for exclusive use on this CRTC
        let plane_claim = drm.claim_plane(primary_plane.handle, drm_surface.crtc())
            .ok_or_else(|| "Failed to claim primary plane")?;

        // Page flip - atomically swap the displayed buffer
        // This presents our rendered frame on screen without tearing
        // The page_flip API expects PlaneState objects
        use smithay::backend::drm::PlaneState;
        match drm_surface.page_flip([PlaneState::from_buffer(&buffer, plane_claim, (0, 0).into())].into_iter(), true) {
            Ok(_) => {},
            Err(e) => {
                info!("Page flip error: {:?}", e);
            }
        }

        // Control frame rate to approximately 60 FPS (16.67ms per frame)
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}