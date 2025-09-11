//! Canvas - Minimal Compositor for Conversational Computer
//! Direct framebuffer approach for simplicity

mod font;
mod input;

use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read, stdout};
use std::os::unix::io::AsRawFd;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    input_text: String,
    cursor_visible: bool,
    cursor_blink_counter: u32,
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
            input_text: String::new(),
            cursor_visible: true,
            cursor_blink_counter: 0,
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
            input_text: String::new(),
            cursor_visible: true,
            cursor_blink_counter: 0,
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

    /// Draw a character at position
    fn draw_char(&mut self, ch: char, x: usize, y: usize, r: u8, g: u8, b: u8, scale: usize) {
        let bitmap = font::get_char(ch);
        
        for row in 0..font::CHAR_HEIGHT {
            for col in 0..font::CHAR_WIDTH {
                if bitmap[row] & (0x80 >> col) != 0 {
                    // Draw scaled pixel
                    for dy in 0..scale {
                        for dx in 0..scale {
                            self.set_pixel(
                                x + col * scale + dx,
                                y + row * scale + dy,
                                r, g, b
                            );
                        }
                    }
                }
            }
        }
    }
    
    /// Draw text at position
    fn draw_text(&mut self, text: &str, x: usize, y: usize, r: u8, g: u8, b: u8, scale: usize) {
        let mut current_x = x;
        for ch in text.chars() {
            self.draw_char(ch, current_x, y, r, g, b, scale);
            current_x += font::CHAR_WIDTH * scale + scale; // Add spacing
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
        
        // Draw "SPOC>" prompt
        self.draw_text("SPOC>", bar_x + 15, bar_y + 22, 100, 200, 255, 2);
        
        // Draw input text
        let text_x = bar_x + 90;
        let input_text = self.input_text.clone();
        self.draw_text(&input_text, text_x, bar_y + 22, 255, 255, 255, 2);
        
        // Draw cursor (vertical bar like modern inputs)
        if self.cursor_visible {
            let cursor_x = text_x + input_text.len() * (font::CHAR_WIDTH * 2 + 2);
            // Draw a thin vertical line
            for y in 0..18 {
                self.set_pixel(cursor_x, bar_y + 21 + y, 255, 255, 255);
                self.set_pixel(cursor_x + 1, bar_y + 21 + y, 255, 255, 255);
            }
        }
    }

    /// Render frame to actual framebuffer
    fn present(&mut self) {
        // Seek to beginning of framebuffer
        use std::io::Seek;
        let _ = self.fb_file.seek(std::io::SeekFrom::Start(0));
        
        // Write in chunks to avoid "no space" error
        let chunk_size = 4096; // Write 4KB at a time
        for chunk in self.framebuffer.chunks(chunk_size) {
            if let Err(e) = self.fb_file.write(chunk) {
                eprintln!("Failed to write to framebuffer: {}", e);
                break;
            }
        }
        let _ = self.fb_file.flush();
    }

    /// Main run loop
    fn run(&mut self) -> io::Result<()> {
        println!("Canvas Compositor Started");
        println!("Resolution: {}x{}", self.width, self.height);
        println!("Press ESC to exit");
        
        // Hide cursor via multiple methods
        print!("\x1b[?25l"); // Hide cursor
        print!("\x1b[2J");   // Clear screen
        print!("\x1b[H");    // Move to home
        let _ = stdout().flush();
        
        // Also try to write to tty directly
        if let Ok(mut tty) = std::fs::OpenOptions::new().write(true).open("/dev/tty") {
            let _ = tty.write_all(b"\x1b[?25l");
            let _ = tty.flush();
        }
        
        // Clear initial text
        self.input_text.clear();
        
        // Setup input handler
        let input_handler = input::InputHandler::new()?;
        
        // Simple event loop
        loop {
            // Handle input
            if let Some(event) = input_handler.poll() {
                match event {
                    input::InputEvent::Char(ch) => {
                        self.input_text.push(ch);
                        self.cursor_visible = true;
                        self.cursor_blink_counter = 0;
                    }
                    input::InputEvent::Backspace => {
                        self.input_text.pop();
                        self.cursor_visible = true;
                        self.cursor_blink_counter = 0;
                    }
                    input::InputEvent::Enter => {
                        // For now, just clear the input
                        self.input_text.clear();
                    }
                    input::InputEvent::Escape => {
                        // Restore cursor before exit
                        print!("\x1b[?25h");
                        let _ = stdout().flush();
                        println!("\nExiting Canvas...");
                        break;
                    }
                }
            }
            
            // Clear to dark blue-gray background
            self.clear(20, 25, 35);
            
            // Draw title at top
            self.draw_text("CANVAS - Conversational Computer", 20, 20, 100, 150, 200, 2);
            self.draw_text("Type and press Enter. ESC to exit.", 20, 45, 80, 120, 160, 1);
            
            // Update cursor blink
            self.cursor_blink_counter += 1;
            if self.cursor_blink_counter > 30 {
                self.cursor_visible = !self.cursor_visible;
                self.cursor_blink_counter = 0;
            }
            
            // Draw SPOC interface
            self.draw_input_bar();
            
            // Present frame
            self.present();
            
            std::thread::sleep(std::time::Duration::from_millis(16)); // 60 FPS
        }
        
        Ok(())
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
            if let Err(e) = canvas.run() {
                eprintln!("Canvas error: {}", e);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to initialize Canvas: {}", e);
            std::process::exit(1);
        }
    }
}