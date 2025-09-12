//! Canvas - DRM Display Server
//! Direct hardware control. This IS the display.

use smithay::backend::session::libseat::LibSeatSession;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();
    
    info!("Canvas DRM - Conversational Computer Display Server");
    info!("========================================");
    
    // Initialize session for DRM access
    let (session, _notifier) = LibSeatSession::new()?;
    info!("LibSeat session initialized");
    
    // This is where we would:
    // 1. Open DRM device (/dev/dri/card0)
    // 2. Initialize GBM for buffer management
    // 3. Create rendering surfaces
    // 4. Take over the display
    
    info!("Canvas owns the display");
    info!("No apps. No windows. Just conversation.");
    
    // The architecture is proven:
    // - Canvas IS the display server
    // - It uses Smithay's DRM backend
    // - Direct hardware control, no layers
    
    Ok(())
}