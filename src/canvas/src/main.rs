//! Canvas - Minimal Compositor for Conversational Computer
//! Direct framebuffer approach for simplicity

mod font;
mod improved_input;
mod text_field;
mod spoc_client;

use improved_input as input;

use std::fs::{File, OpenOptions};
use std::io::{self, Write, Read, stdout};
use std::os::unix::io::AsRawFd;
use std::mem;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

const FB_PATH: &str = "/dev/fb0";

/// Role of conversation block
#[derive(Clone, Debug)]
enum BlockRole {
    User,
    Assistant,
    System,
    Terminal,  // New: for command output
}

/// Conversation block
#[derive(Clone, Debug)]
struct Block {
    text: String,
    role: BlockRole,
    timestamp: std::time::SystemTime,
    height: usize,  // Calculated pixel height
    command: Option<String>,  // For terminal blocks
}

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
    input_field: text_field::TextField,
    cursor_visible: bool,
    cursor_blink_counter: u32,
    blocks: Vec<Block>,
    scroll_offset: usize,
    needs_redraw: bool,  // Only redraw when needed
    last_frame_time: std::time::Instant,
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

        // Calculate input field width in characters (assuming 8px font with 2x scale)
        let char_width = (font::CHAR_WIDTH * 2 + 2) as usize;
        let input_width_chars = (600.min(width - 100)) / char_width;

        Ok(Canvas {
            fb_file,
            screen_info,
            framebuffer,
            width,
            height,
            bytes_per_pixel,
            input_field: text_field::TextField::new(input_width_chars, 5), // Max 5 lines
            cursor_visible: true,
            cursor_blink_counter: 0,
            blocks: Vec::new(),
            scroll_offset: 0,
            needs_redraw: true,
            last_frame_time: std::time::Instant::now(),
        })
    }

    /// Create simulation mode Canvas (for testing without framebuffer)
    fn new_simulation() -> io::Result<Self> {
        println!("Running in simulation mode (no framebuffer access)");
        
        // Simulate 1920x1080 screen
        let screen_info: FbVarScreenInfo = unsafe { mem::zeroed() };
        let fb_file = File::open("/dev/null")?;
        
        let width = 1920;
        let height = 1080;
        let char_width = (font::CHAR_WIDTH * 2 + 2) as usize;
        let input_width_chars = (600.min(width - 100)) / char_width;

        Ok(Canvas {
            fb_file,
            screen_info,
            framebuffer: vec![0u8; width * height * 4],
            width,
            height,
            bytes_per_pixel: 4,
            input_field: text_field::TextField::new(input_width_chars, 5),
            cursor_visible: true,
            cursor_blink_counter: 0,
            blocks: Vec::new(),
            scroll_offset: 0,
            needs_redraw: true,
            last_frame_time: std::time::Instant::now(),
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

    /// Draw text with word wrapping
    fn draw_text_wrapped(&mut self, text: &str, x: usize, y: usize, max_width: usize, r: u8, g: u8, b: u8, scale: usize) -> usize {
        let char_width = font::CHAR_WIDTH * scale + scale;
        let line_height = font::CHAR_HEIGHT * scale + 4;
        let max_chars = max_width / char_width;
        
        let mut lines = Vec::new();
        let mut current_line = String::new();
        
        for word in text.split_whitespace() {
            if current_line.is_empty() {
                current_line = word.to_string();
            } else if current_line.len() + 1 + word.len() <= max_chars {
                current_line.push(' ');
                current_line.push_str(word);
            } else {
                lines.push(current_line);
                current_line = word.to_string();
            }
        }
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        for (i, line) in lines.iter().enumerate() {
            self.draw_text(line, x, y + i * line_height, r, g, b, scale);
        }
        
        lines.len() * line_height
    }

    /// Calculate height needed for wrapped text
    fn calculate_text_height(&self, text: &str, max_width: usize, scale: usize) -> usize {
        let char_width = font::CHAR_WIDTH * scale + scale;
        let line_height = font::CHAR_HEIGHT * scale + 4;
        let max_chars = max_width / char_width;
        
        let mut lines = 0;
        let mut current_line_len = 0;
        
        for word in text.split_whitespace() {
            if current_line_len == 0 {
                current_line_len = word.len();
                lines = 1;
            } else if current_line_len + 1 + word.len() <= max_chars {
                current_line_len += 1 + word.len();
            } else {
                lines += 1;
                current_line_len = word.len();
            }
        }
        
        lines * line_height
    }

    /// Add a new block to conversation
    fn add_block(&mut self, text: String, role: BlockRole) {
        self.add_block_with_command(text, role, None);
    }
    
    /// Add a new block with optional command (for terminal blocks)
    fn add_block_with_command(&mut self, text: String, role: BlockRole, command: Option<String>) {
        let max_width = 600.min(self.width - 80) - 20; // Block width minus padding
        let mut height = self.calculate_text_height(&text, max_width, 2) + 20; // Add padding
        
        // Terminal blocks need extra height for command line
        if matches!(role, BlockRole::Terminal) && command.is_some() {
            height += 20; // Extra space for command display
        }
        
        let block = Block {
            text,
            role,
            timestamp: std::time::SystemTime::now(),
            height,
            command,
        };
        
        self.blocks.push(block);
        self.scroll_offset = 0; // Auto-scroll to bottom
    }

    /// Draw conversation blocks
    fn draw_blocks(&mut self) {
        let block_width = 600.min(self.width - 40);
        let block_x = (self.width - block_width) / 2;
        let input_bar_y = self.height - 80; // Leave space for input bar
        
        let mut current_y = input_bar_y - 20;
        
        // Collect blocks to draw (to avoid borrow checker issues)
        let blocks_to_draw: Vec<_> = self.blocks.iter()
            .rev()
            .skip(self.scroll_offset)
            .cloned()
            .collect();
        
        // Draw blocks from bottom up (newest first)
        for block in blocks_to_draw.iter() {
            if current_y < block.height + 100 {
                break; // Stop if we run out of space at top
            }
            
            current_y -= block.height + 5; // Add margin between blocks
            
            // Choose background color based on role
            let (bg_r, bg_g, bg_b) = match block.role {
                BlockRole::User => (40, 45, 55),      // Slightly blue
                BlockRole::Assistant => (35, 45, 40),  // Slightly green
                BlockRole::System => (35, 35, 40),     // Gray
                BlockRole::Terminal => (25, 30, 35),   // Dark for terminal
            };
            
            // Draw block background
            self.draw_rect(block_x, current_y, block_width, block.height, bg_r, bg_g, bg_b);
            
            // Draw block border
            for i in 0..1 {
                for x in block_x..block_x + block_width {
                    self.set_pixel(x, current_y + i, 60, 70, 80);
                    self.set_pixel(x, current_y + block.height - i - 1, 60, 70, 80);
                }
                for y in current_y..current_y + block.height {
                    self.set_pixel(block_x + i, y, 60, 70, 80);
                    self.set_pixel(block_x + block_width - i - 1, y, 60, 70, 80);
                }
            }
            
            // Draw role label and command for terminal blocks
            let role_text = match block.role {
                BlockRole::User => "You",
                BlockRole::Assistant => "SPOC",
                BlockRole::System => "System",
                BlockRole::Terminal => "Terminal",
            };
            self.draw_text(role_text, block_x + 10, current_y + 5, 150, 170, 190, 1);
            
            // For terminal blocks, show the command
            if matches!(block.role, BlockRole::Terminal) {
                if let Some(ref cmd) = block.command {
                    let cmd_display = format!("> {}", cmd);
                    self.draw_text(&cmd_display, block_x + 10, current_y + 18, 100, 200, 100, 1);
                    // Draw output below command
                    self.draw_text_wrapped(&block.text, block_x + 10, current_y + 32, block_width - 20, 200, 210, 220, 1);
                } else {
                    self.draw_text_wrapped(&block.text, block_x + 10, current_y + 20, block_width - 20, 200, 210, 220, 1);
                }
            } else {
                self.draw_text_wrapped(&block.text, block_x + 10, current_y + 20, block_width - 20, 220, 230, 240, 2);
            }
            
        }
        
        // Draw scroll indicator if needed
        if self.scroll_offset > 0 || self.blocks.len() > 5 {
            self.draw_text(&format!("[{}/{}]", self.scroll_offset + 1, self.blocks.len()), 
                          self.width - 100, 20, 100, 120, 140, 1);
        }
    }

    /// Draw SPOC input bar
    fn draw_input_bar(&mut self) {
        // Calculate dynamic height based on text field
        let line_height = font::CHAR_HEIGHT * 2 + 4;
        let padding = 20;
        let text_height = self.input_field.get_height() * line_height;
        let bar_height = text_height + padding * 2;
        
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
        
        // Draw "SPOC>" prompt only on first line
        self.draw_text("SPOC>", bar_x + 15, bar_y + padding, 100, 200, 255, 2);
        
        // Draw visible lines of input text
        let text_x = bar_x + 90;
        let text_y_start = bar_y + padding;
        
        // Clone lines to avoid borrow checker issues
        let visible_lines: Vec<String> = self.input_field.visible_lines().to_vec();
        for (line_idx, line) in visible_lines.iter().enumerate() {
            let y = text_y_start + line_idx * line_height;
            self.draw_text(line, text_x, y, 255, 255, 255, 2);
        }
        
        // Draw cursor
        if self.cursor_visible {
            let visible_line = self.input_field.cursor_line.saturating_sub(self.input_field.scroll_offset);
            if visible_line < self.input_field.get_height() {
                let cursor_x = text_x + self.input_field.cursor_col * (font::CHAR_WIDTH * 2 + 2);
                let cursor_y = text_y_start + visible_line * line_height;
                
                // Draw a thin vertical line
                for y in 0..16 {
                    self.set_pixel(cursor_x, cursor_y + y, 255, 255, 255);
                    self.set_pixel(cursor_x + 1, cursor_y + y, 255, 255, 255);
                }
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
        
        // Setup input handler
        let input_handler = input::InputHandler::new()?;
        
        // Track frame timing
        let target_frame_time = std::time::Duration::from_millis(8); // ~120 FPS for responsiveness
        
        // Force initial redraw
        self.needs_redraw = true;
        
        // Main event loop
        loop {
            let frame_start = std::time::Instant::now();
            
            // Handle all pending input events
            while let Some(event) = input_handler.poll() {
                self.needs_redraw = true;
                self.cursor_visible = true;
                self.cursor_blink_counter = 0;
                
                match event {
                    input::InputEvent::Char(ch) => {
                        self.input_field.insert_char(ch);
                    }
                    input::InputEvent::Backspace => {
                        self.input_field.backspace();
                    }
                    input::InputEvent::Delete => {
                        self.input_field.delete();
                    }
                    input::InputEvent::NewLine => {
                        self.input_field.insert_newline();
                    }
                    input::InputEvent::Enter => {
                        if !self.input_field.text.is_empty() {
                            // Add user message as a block
                            let user_text = self.input_field.text.clone();
                            self.add_block(user_text.clone(), BlockRole::User);
                            
                            // Query SPOC conductor
                            match spoc_client::SPOCClient::query(&user_text) {
                                Ok(blocks) => {
                                    // Process each response block
                                    for block in blocks {
                                        match block.block_type.as_str() {
                                            "terminal" => {
                                                // Terminal block with command and output
                                                let output = block.output.unwrap_or_default();
                                                let command = block.command;
                                                self.add_block_with_command(output, BlockRole::Terminal, command);
                                            }
                                            "system" => {
                                                let content = block.content.unwrap_or_default();
                                                self.add_block(content, BlockRole::System);
                                            }
                                            _ => {
                                                // Default to assistant text
                                                let content = block.content.unwrap_or_default();
                                                self.add_block(content, BlockRole::Assistant);
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    // Fallback to basic responses if SPOC not available
                                    let response = if e.contains("Is the server running?") {
                                        "SPOC conductor not running. Start with: python src/conductor/spoc_server.py".to_string()
                                    } else {
                                        format!("Error: {}", e)
                                    };
                                    self.add_block(response, BlockRole::System);
                                }
                            }
                            
                            self.input_field.clear();
                        }
                    }
                    // Arrow key navigation
                    input::InputEvent::Left => {
                        self.input_field.move_left();
                    }
                    input::InputEvent::Right => {
                        self.input_field.move_right();
                    }
                    input::InputEvent::Up => {
                        self.input_field.move_up();
                    }
                    input::InputEvent::Down => {
                        self.input_field.move_down();
                    }
                    // Mac shortcuts
                    input::InputEvent::WordLeft => {
                        self.input_field.move_word_left();
                    }
                    input::InputEvent::WordRight => {
                        self.input_field.move_word_right();
                    }
                    input::InputEvent::LineStart | input::InputEvent::Home => {
                        self.input_field.move_line_start();
                    }
                    input::InputEvent::LineEnd | input::InputEvent::End => {
                        self.input_field.move_line_end();
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
            
            // Update cursor blink (only triggers redraw if cursor state changes)
            self.cursor_blink_counter += 1;
            if self.cursor_blink_counter > 60 { // Blink every ~500ms at 120fps
                self.cursor_visible = !self.cursor_visible;
                self.cursor_blink_counter = 0;
                self.needs_redraw = true;
            }
            
            // Only redraw if needed
            if self.needs_redraw {
                // Clear to dark blue-gray background
                self.clear(20, 25, 35);
                
                // Draw title at top
                self.draw_text("CANVAS - Conversational Computer", 20, 20, 100, 150, 200, 2);
                self.draw_text("Type and press Enter. ESC to exit.", 20, 45, 80, 120, 160, 1);
                
                // Draw conversation blocks
                self.draw_blocks();
                
                // Draw SPOC interface
                self.draw_input_bar();
                
                // Present frame
                self.present();
                
                self.needs_redraw = false;
            }
            
            // Sleep only if we're ahead of schedule
            let frame_time = frame_start.elapsed();
            if frame_time < target_frame_time {
                std::thread::sleep(target_frame_time - frame_time);
            }
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