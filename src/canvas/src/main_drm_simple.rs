//! Canvas - Simple DRM test without session management
//! Direct hardware control for testing

use std::{
    fs::OpenOptions,
    os::unix::io::IntoRawFd,
};

use smithay::{
    backend::{
        allocator::gbm::GbmDevice,
        drm::{DrmDevice, DrmDeviceFd},
        egl::EGLDevice,
        renderer::gles::GlesRenderer,
    },
    reexports::{
        drm::control::{connector, Device as ControlDevice},
    },
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
    
    info!("Canvas Simple DRM Test");
    info!("======================");
    
    // Open DRM device directly
    let gpu_path = "/dev/dri/card0";
    info!("Opening DRM device: {}", gpu_path);
    
    let gpu_file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(gpu_path)?;
    
    let fd = gpu_file.into_raw_fd();
    let drm_fd = DrmDeviceFd::new_from_fd(fd)?;
    
    info!("DRM device opened successfully");
    
    // Create DRM device
    let (drm, _notifier) = DrmDevice::new(drm_fd.clone(), true)?;
    info!("DRM device created");
    
    // Create GBM device
    let gbm = GbmDevice::new(drm_fd.clone())?;
    info!("GBM device created");
    
    // Create EGL device and renderer
    let egl = EGLDevice::new(gbm.clone())?;
    let renderer = unsafe { GlesRenderer::new(egl.create_context()?)? };
    info!("GLES renderer created");
    
    // Get device info
    let res = drm_fd.resource_handles()?;
    info!("Found {} connectors", res.connectors().len());
    
    // Find connected outputs
    for connector_handle in res.connectors() {
        let connector = drm_fd.get_connector(*connector_handle, false)?;
        info!("Connector {:?}: {:?}", connector.interface(), connector.state());
        
        if connector.state() == connector::State::Connected {
            info!("  Connected! Modes available: {}", connector.modes().len());
            if let Some(mode) = connector.modes().first() {
                info!("  First mode: {}x{} @ {}Hz", 
                    mode.size().0, 
                    mode.size().1,
                    mode.vrefresh()
                );
            }
        }
    }
    
    info!("Test complete - DRM/GBM stack is working!");
    
    Ok(())
}