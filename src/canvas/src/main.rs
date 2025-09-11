//! Canvas - Minimal Compositor for Conversational Computer
//! Direct framebuffer approach for simplicity

use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read};
use std::os::unix::io::AsRawFd;
use std::mem;
use std::ptr;

const FB_PATH: &str = "/dev/fb0";

/// Framebuffer information structure
#[repr(C)]
struct FbVarScreenInfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

/// Canvas compositor
struct Canvas {
    fb_file: File,
    screen_info: FbVarScreenInfo,
    framebuffer: Vec<u8>,
    width: usize,
    height: usize,
    bytes_per_pixel: usize,
}

impl Canvas {
    /// Create new Canvas instance
    fn new() -> io::Result<Self> {
        // Try to open framebuffer
        let fb_file = match OpenOptions::new()
            .read(true)
            .write(true)
            .open(FB_PATH) 
        {
            Ok(f) => f,
            Err(_) => {
                println!("Cannot open framebuffer. Running in simulation mode.");
                return Self::new_simulation();
            }
        };

        // Get screen info via ioctl
        let mut screen_info: FbVarScreenInfo = unsafe { mem::zeroed() };
        
        // FBIOGET_VSCREENINFO = 0x4600
        let result = unsafe {
            libc::ioctl(fb_file.as_raw_fd(), 0x4600, &mut screen_info)
        };
        
        if result < 0 {
            return Self::new_simulation();
        }

        let width = screen_info.xres as usize;
        let height = screen_info.yres as usize;
        let bytes_per_pixel = (screen_info.bits_per_pixel / 8) as usize;
        let framebuffer = vec![0u8; width * height * bytes_per_pixel];

        Ok(Canvas {
            fb_file,
            screen_info,
            framebuffer,
            width,
            height,
            bytes_per_pixel,
        })
    }

    /// Create simulation mode Canvas (for testing without framebuffer)
    fn new_simulation() -> io::Result<Self> {
        println!("Running in simulation mode (no framebuffer access)");
        
        // Simulate 1920x1080 screen
        let screen_info: FbVarScreenInfo = unsafe { mem::zeroed() };
        let fb_file = File::open("/dev/null")?;
        
        Ok(Canvas {
            fb_file,
            screen_info,
            framebuffer: vec![0u8; 1920 * 1080 * 4],
            width: 1920,
            height: 1080,
            bytes_per_pixel: 4,
        })
    }

    /// Clear screen to background color
    fn clear(&mut self, r: u8, g: u8, b: u8) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.set_pixel(x, y, r, g, b);
            }
        }
    }

    /// Set a pixel color
    fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        if x >= self.width || y >= self.height {
            return;
        }

        let offset = (y * self.width + x) * self.bytes_per_pixel;
        
        if self.bytes_per_pixel == 4 {
            // BGRA format (most common)
            self.framebuffer[offset] = b;
            self.framebuffer[offset + 1] = g;
            self.framebuffer[offset + 2] = r;
            self.framebuffer[offset + 3] = 255; // Alpha
        } else if self.bytes_per_pixel == 3 {
            // BGR format
            self.framebuffer[offset] = b;
            self.framebuffer[offset + 1] = g;
            self.framebuffer[offset + 2] = r;
        }
    }

    /// Draw a rectangle
    fn draw_rect(&mut self, x: usize, y: usize, w: usize, h: usize, r: u8, g: u8, b: u8) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(x + dx, y + dy, r, g, b);
            }
        }
    }

    /// Draw SPOC input bar
    fn draw_input_bar(&mut self) {
        // Draw input bar background at bottom
        let bar_height = 60;
        let bar_y = self.height - bar_height - 20;
        let bar_width = 600.min(self.width - 40);
        let bar_x = (self.width - bar_width) / 2;
        
        // Dark background for input
        self.draw_rect(bar_x, bar_y, bar_width, bar_height, 30, 35, 45);
        
        // Border
        for i in 0..2 {
            // Top border
            for x in bar_x..bar_x + bar_width {
                self.set_pixel(x, bar_y + i, 100, 150, 255);
            }
            // Bottom border
            for x in bar_x..bar_x + bar_width {
                self.set_pixel(x, bar_y + bar_height - i - 1, 100, 150, 255);
            }
            // Left border
            for y in bar_y..bar_y + bar_height {
                self.set_pixel(bar_x + i, y, 100, 150, 255);
            }
            // Right border
            for y in bar_y..bar_y + bar_height {
                self.set_pixel(bar_x + bar_width - i - 1, y, 100, 150, 255);
            }
        }
        
        // Draw "SPOC>" prompt (simplified - just a colored box for now)
        self.draw_rect(bar_x + 10, bar_y + 20, 50, 20, 150, 150, 150);
    }

    /// Render frame to actual framebuffer
    fn present(&mut self) {
        // Write to framebuffer
        if let Err(e) = self.fb_file.write_all(&self.framebuffer) {
            eprintln!("Failed to write to framebuffer: {}", e);
        }
        let _ = self.fb_file.flush();
    }

    /// Main run loop
    fn run(&mut self) {
        println!("Canvas Compositor Started");
        println!("Resolution: {}x{}", self.width, self.height);
        println!("Press Ctrl+C to exit");
        
        // Clear to dark blue-gray background
        self.clear(20, 25, 35);
        
        // Draw SPOC interface
        self.draw_input_bar();
        
        // Present frame
        self.present();
        
        // Simple event loop
        loop {
            std::thread::sleep(std::time::Duration::from_millis(16)); // 60 FPS
            
            // In real implementation, we'd handle input events here
            // For now, just keep running until Ctrl+C
        }
    }
}

fn main() {
    println!("Canvas - Conversational Computer Compositor");
    println!("==========================================");
    
    // Check if running as root (needed for framebuffer access)
    if unsafe { libc::geteuid() } != 0 {
        println!("Note: Running without root. Will use simulation mode.");
        println!("For real framebuffer access, run with sudo from TTY.");
    }
    
    // Create and run Canvas
    match Canvas::new() {
        Ok(mut canvas) => {
            canvas.run();
        }
        Err(e) => {
            eprintln!("Failed to initialize Canvas: {}", e);
            std::process::exit(1);
        }
    }
}