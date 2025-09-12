use std::time::Instant;

pub struct SpocInput {
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    animation_start: Instant,
    is_focused: bool,
    scroll_offset: usize,
}

impl SpocInput {
    pub fn new() -> Self {
        Self {
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            animation_start: Instant::now(),
            is_focused: true,
            scroll_offset: 0,
        }
    }
    
    pub fn add_char(&mut self, c: char) {
        if c == '\n' {
            // Split current line at cursor
            let current_line = self.lines[self.cursor_line].clone();
            let (before, after) = current_line.split_at(self.cursor_col);
            
            self.lines[self.cursor_line] = before.to_string();
            self.lines.insert(self.cursor_line + 1, after.to_string());
            
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.lines[self.cursor_line].insert(self.cursor_col, c);
            self.cursor_col += 1;
        }
    }
    
    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            self.lines[self.cursor_line].remove(self.cursor_col - 1);
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            // Join with previous line
            let current = self.lines.remove(self.cursor_line);
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
            self.lines[self.cursor_line].push_str(&current);
        }
    }
    
    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
        }
    }
    
    pub fn move_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
        }
    }
    
    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        } else if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.lines[self.cursor_line].len();
        }
    }
    
    pub fn move_right(&mut self) {
        if self.cursor_col < self.lines[self.cursor_line].len() {
            self.cursor_col += 1;
        } else if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }
    
    pub fn get_text(&self) -> String {
        self.lines.join("\n")
    }
    
    pub fn clear(&mut self) {
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
    }
    
    pub fn render(&self, buffer: &mut [u32], width: usize, height: usize) {
        // Spotlight-like aesthetics: centered, glass effect, minimal
        
        // Input window dimensions (like Spotlight but taller for multiline)
        let window_width = 700.min(width - 100);
        let window_height = 200.min(height / 3);
        let window_x = (width - window_width) / 2;
        let window_y = height / 3;
        
        // Animation (fade in)
        let progress = (self.animation_start.elapsed().as_millis() as f32 / 150.0).min(1.0);
        
        // Draw glass background with rounded corners effect
        for y in 0..window_height {
            for x in 0..window_width {
                let px = window_x + x;
                let py = window_y + y;
                
                if px < width && py < height {
                    // Rounded corners check
                    let corner_radius = 15;
                    let is_corner = (x < corner_radius || x > window_width - corner_radius) && 
                                   (y < corner_radius || y > window_height - corner_radius);
                    
                    if !is_corner || Self::in_circle(x, y, corner_radius, window_width, window_height) {
                        let idx = py * width + px;
                        
                        // Glass effect - semi-transparent dark with blur
                        let existing = buffer[idx];
                        let r = ((existing >> 16) & 0xFF) as u8;
                        let g = ((existing >> 8) & 0xFF) as u8;
                        let b = (existing & 0xFF) as u8;
                        
                        // Darken and blur effect
                        let glass_r = (r as f32 * 0.3 + 20.0) as u8;
                        let glass_g = (g as f32 * 0.3 + 20.0) as u8;
                        let glass_b = (b as f32 * 0.3 + 25.0) as u8;
                        
                        let alpha = (progress * 240.0) as u8;
                        buffer[idx] = (alpha as u32) << 24 | ((glass_r as u32) << 16) | ((glass_g as u32) << 8) | (glass_b as u32);
                    }
                }
            }
        }
        
        // Draw subtle border
        for x in 10..(window_width - 10) {
            let px = window_x + x;
            if px < width {
                // Top border
                let py_top = window_y + 10;
                if py_top < height {
                    buffer[py_top * width + px] = 0xFF3A3A4A;
                }
                // Bottom border
                let py_bottom = window_y + window_height - 10;
                if py_bottom < height {
                    buffer[py_bottom * width + px] = 0xFF3A3A4A;
                }
            }
        }
        
        // Draw text lines
        let text_x = window_x + 30;
        let text_y = window_y + 30;
        let line_height = 20;
        let max_visible_lines = (window_height - 60) / line_height;
        
        for (i, line) in self.lines.iter().enumerate().skip(self.scroll_offset).take(max_visible_lines) {
            let y = text_y + (i - self.scroll_offset) * line_height;
            self.draw_text(buffer, width, line, text_x, y, 0xFFE0E0E0);
        }
        
        // Draw cursor (blinking)
        if self.is_focused && (self.animation_start.elapsed().as_millis() / 500) % 2 == 0 {
            let cursor_y = text_y + (self.cursor_line - self.scroll_offset) * line_height;
            let cursor_x = text_x + self.cursor_col * 8; // Approximate char width
            
            if cursor_x < window_x + window_width - 30 && cursor_y < window_y + window_height - 30 {
                for y in 0..16 {
                    let py = cursor_y + y;
                    if cursor_x < width && py < height {
                        buffer[py * width + cursor_x] = 0xFFFFFFFF;
                        buffer[py * width + cursor_x + 1] = 0xFFFFFFFF; // 2px wide cursor
                    }
                }
            }
        }
        
        // Draw placeholder text if empty
        if self.lines.len() == 1 && self.lines[0].is_empty() {
            let placeholder = "Talk to SPOC...";
            self.draw_text(buffer, width, placeholder, text_x, text_y, 0xFF606070);
        }
        
        // Draw subtle hint at bottom
        let hint = "Enter to send • Shift+Enter for new line • Esc to close";
        let hint_x = window_x + (window_width - hint.len() * 6) / 2;
        let hint_y = window_y + window_height - 25;
        self.draw_text(buffer, width, hint, hint_x, hint_y, 0xFF505060);
    }
    
    fn in_circle(x: usize, y: usize, radius: usize, width: usize, height: usize) -> bool {
        let corner_x = if x < radius { x } else if x > width - radius { width - x } else { return true };
        let corner_y = if y < radius { y } else if y > height - radius { height - y } else { return true };
        
        (corner_x * corner_x + corner_y * corner_y) < (radius * radius)
    }
    
    fn draw_text(&self, buffer: &mut [u32], width: usize, text: &str, x: usize, y: usize, color: u32) {
        // Simple text rendering placeholder
        for (i, _ch) in text.chars().enumerate() {
            let px = x + (i * 8);
            if px < width - 8 && y < buffer.len() / width - 8 {
                // Draw simple pixels for each character (would use proper font rendering)
                for dy in 0..10 {
                    for dx in 0..6 {
                        let idx = (y + dy) * width + (px + dx);
                        if idx < buffer.len() {
                            // Anti-aliased effect
                            if dy == 0 || dy == 9 || dx == 0 || dx == 5 {
                                // Lighter edges
                                let dimmed = (color & 0x00FFFFFF) | 0x80000000;
                                buffer[idx] = dimmed;
                            } else {
                                buffer[idx] = color;
                            }
                        }
                    }
                }
            }
        }
    }
}