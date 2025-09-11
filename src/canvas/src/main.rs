//! Canvas - Minimal Wayland Compositor for Conversational Computer

use std::time::Instant;

fn main() {
    println!("Canvas Compositor");
    println!("=================");
    println!("Conversational Computer - AI-first interaction");
    println!("");
    
    // For now, just a simple event loop
    // Full Smithay integration requires more complex setup
    let start = Instant::now();
    
    loop {
        // Simulate frame rendering
        std::thread::sleep(std::time::Duration::from_millis(16)); // 60 FPS
        
        // Exit after 5 seconds for testing
        if start.elapsed().as_secs() > 5 {
            println!("Test complete - Canvas would run here");
            break;
        }
    }
    
    println!("Canvas terminated");
}