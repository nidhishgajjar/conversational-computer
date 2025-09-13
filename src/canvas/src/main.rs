//! Canvas - Minimal DRM Display Control
//! Only 100% working code - no janky rendering

use std::{
    fs::OpenOptions,
    os::unix::io::{IntoRawFd, OwnedFd, FromRawFd},
};

use smithay::{
    backend::{
        allocator::gbm::GbmDevice,
        drm::{DrmDevice, DrmDeviceFd},
        egl::{EGLDisplay, EGLContext},
        renderer::gles::GlesRenderer,
    },
    reexports::drm::control::{connector, Device as ControlDevice},
    utils::DeviceFd,
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

    // Create DRM device
    let (_drm, _notifier) = DrmDevice::new(drm_fd.clone(), false)?;
    info!("✓ DRM device created");

    // Create GBM device
    let gbm = GbmDevice::new(drm_fd.clone())?;
    info!("✓ GBM device created");

    // Create EGL context and renderer
    let egl_display = unsafe { EGLDisplay::new(gbm.clone())? };
    let egl_context = EGLContext::new(&egl_display)?;
    let _renderer = unsafe { GlesRenderer::new(egl_context)? };
    info!("✓ GLES renderer created");

    // Display detection
    let res = drm_fd.resource_handles()?;
    info!("Found {} connectors", res.connectors().len());

    for connector_handle in res.connectors() {
        let connector = drm_fd.get_connector(*connector_handle, false)?;

        if connector.state() == connector::State::Connected {
            info!("✓ Connected display: {:?}", connector.interface());

            if let Some(mode) = connector.modes().first() {
                info!("  Mode: {}x{} @ {}Hz",
                    mode.size().0, mode.size().1, mode.vrefresh());
            }
        }
    }

    info!("");
    info!("Display initialized successfully");
    info!("Removed broken rendering code - keeping only working initialization");

    // Just keep the process running
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}