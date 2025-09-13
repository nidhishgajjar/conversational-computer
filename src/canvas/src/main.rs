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
    // SCANOUT flag: buffer can be displayed on screen (required for DRM)
    let mut allocator = GbmAllocator::new(
        gbm.clone(),
        GbmBufferFlags::RENDERING | GbmBufferFlags::SCANOUT,
    );
    info!("✓ GBM allocator created (RENDERING | SCANOUT)");

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

    let (drm_surface, _connector) = surface
        .ok_or_else(|| "No suitable display found")?;
    let display_mode = mode.ok_or_else(|| "No display mode found")?;

    info!("");
    info!("Starting minimal render loop - clearing with solid color");
    info!("Display mode: {}x{} @ {}Hz", display_mode.size().0, display_mode.size().1, display_mode.vrefresh());
    info!("Press Ctrl+C to exit");

    // Try to create a buffer once to test if it works
    let size = Size::from((display_mode.size().0 as i32, display_mode.size().1 as i32));

    // First, let's try to understand what formats are supported
    info!("Attempting to create buffer with size: {}x{}", size.w, size.h);

    // Try different formats if XRGB8888 doesn't work
    let test_formats = [
        Fourcc::Xrgb8888,
        Fourcc::Argb8888,
        Fourcc::Rgb565,
        Fourcc::Xbgr8888,
    ];

    // Import DrmModifier for LINEAR
    use smithay::reexports::gbm::Modifier;

    let mut working_format = None;
    for format in &test_formats {
        // First try with no modifiers (implicit modifier)
        match allocator.create_buffer(size.w as u32, size.h as u32, *format, &[]) {
            Ok(buffer) => {
                info!("✓ Successfully created buffer with format: {:?} (no modifiers)", format);
                working_format = Some((*format, None));
                // Clean up test buffer
                drop(buffer);
                break;
            }
            Err(e) => {
                info!("  Failed with format {:?} (no modifiers): {:?}", format, e);

                // Try with LINEAR modifier for better compatibility
                match allocator.create_buffer(size.w as u32, size.h as u32, *format, &[Modifier::Linear]) {
                    Ok(buffer) => {
                        info!("✓ Successfully created buffer with format: {:?} (LINEAR modifier)", format);
                        working_format = Some((*format, Some(Modifier::Linear)));
                        // Clean up test buffer
                        drop(buffer);
                        break;
                    }
                    Err(e) => {
                        info!("  Failed with format {:?} (LINEAR modifier): {:?}", format, e);
                    }
                }
            }
        }
    }

    let (format, modifier) = working_format.ok_or("No supported buffer format found")?;
    info!("Using format: {:?} with modifier: {:?} for rendering", format, modifier);

    // Create two buffers for double buffering
    info!("Creating double buffers for page flipping...");

    let create_buffer = |allocator: &mut GbmAllocator<_>| -> Result<_, Box<dyn std::error::Error>> {
        if let Some(mod_value) = modifier {
            allocator
                .create_buffer(size.w as u32, size.h as u32, format, &[mod_value])
                .map_err(|e| format!("Failed to create buffer with modifier: {:?}", e).into())
        } else {
            allocator
                .create_buffer(size.w as u32, size.h as u32, format, &[])
                .map_err(|e| format!("Failed to create buffer: {:?}", e).into())
        }
    };

    let buffer1 = create_buffer(&mut allocator)?;
    let buffer2 = create_buffer(&mut allocator)?;

    // Create framebuffers for both buffers
    use smithay::reexports::drm::control::FbCmd2Flags;
    let fb1 = drm_fd.add_planar_framebuffer(&buffer1, FbCmd2Flags::empty())
        .map_err(|e| format!("Failed to create framebuffer 1: {:?}", e))?;
    let fb2 = drm_fd.add_planar_framebuffer(&buffer2, FbCmd2Flags::empty())
        .map_err(|e| format!("Failed to create framebuffer 2: {:?}", e))?;

    info!("✓ Double buffers created successfully");

    let buffers = [(buffer1, fb1), (buffer2, fb2)];
    let mut current_buffer = 0;

    // Simple render loop - creates frames and displays them
    let mut frame_count = 0u64;
    loop {
        frame_count += 1;

        // Get current buffer and framebuffer
        let (buffer, fb_handle) = &buffers[current_buffer];

        // Try direct rendering to the GBM buffer instead of exporting to Dmabuf
        // This should be more direct and work better with the display

        // Bind the buffer directly as render target
        let render_result = renderer.bind(buffer);

        match render_result {
            Ok(mut target) => {
                // Direct rendering to GBM buffer succeeded
                let mut frame = renderer.render(&mut target, size, Transform::Normal)
                    .map_err(|e| format!("Failed to start frame: {:?}", e))?;

                // Use a bright blue color to make it obvious
                let blue_value = 0.5 + (frame_count as f32 * 0.02).sin().abs() * 0.5;
                frame.clear([0.0, 0.2, blue_value, 1.0].into(), &[])
                    .map_err(|e| format!("Failed to clear frame: {:?}", e))?;

                let sync = frame.finish()
                    .map_err(|e| format!("Failed to finish frame: {:?}", e))?;

                // Wait for rendering to complete before page flip
                sync.wait();
            }
            Err(_) => {
                // Fallback: Export to Dmabuf if direct binding fails
                let mut dmabuf: Dmabuf = buffer.export()
                    .map_err(|e| format!("Failed to export buffer: {:?}", e))?;

                let mut target = renderer.bind(&mut dmabuf)
                    .map_err(|e| format!("Failed to bind dmabuf: {:?}", e))?;

                let mut frame = renderer.render(&mut target, size, Transform::Normal)
                    .map_err(|e| format!("Failed to start frame: {:?}", e))?;

                // Use red color for dmabuf path so we know which path is taken
                frame.clear([0.5, 0.0, 0.0, 1.0].into(), &[])
                    .map_err(|e| format!("Failed to clear frame: {:?}", e))?;

                let sync = frame.finish()
                    .map_err(|e| format!("Failed to finish frame: {:?}", e))?;

                // Wait for rendering to complete
                sync.wait();
            }
        }

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
        use smithay::backend::drm::{PlaneState, PlaneConfig};
        use smithay::utils::Rectangle;

        // Create PlaneState with proper configuration
        let plane_state = PlaneState {
            handle: plane_claim.plane(),
            config: Some(PlaneConfig {
                src: Rectangle::from_size((size.w as f64, size.h as f64).into()),
                dst: Rectangle::from_size((size.w, size.h).into()),
                transform: Transform::Normal,
                alpha: 1.0,
                damage_clips: None,
                fb: *fb_handle,  // Use pre-created framebuffer
                fence: None,
            }),
        };

        match drm_surface.page_flip([plane_state].into_iter(), true) {
            Ok(_) => {
                // Successfully flipped - frame is now displayed
                if frame_count % 60 == 0 {
                    info!("Frame {}: Page flip successful", frame_count);
                }

                // Switch to the other buffer for next frame
                current_buffer = (current_buffer + 1) % 2;
            },
            Err(e) => {
                info!("Frame {}: Page flip error: {:?}", frame_count, e);
                // Don't switch buffers on error, try same buffer again
            }
        }

        // Control frame rate to approximately 60 FPS (16.67ms per frame)
        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}