//! Multiline text field with wrapping and cursor management

use crate::font;

#[derive(Debug, Clone)]
pub struct TextField {
    pub text: String,
    pub cursor_pos: usize,
    pub lines: Vec<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub width: usize,  // Width in characters
    pub max_lines: usize,
    pub scroll_offset: usize,
}

impl TextField {
    pub fn new(width: usize, max_lines: usize) -> Self {
        TextField {
            text: String::new(),
            cursor_pos: 0,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            width,
            max_lines,
            scroll_offset: 0,
        }
    }
    
    /// Insert character at cursor position
    pub fn insert_char(&mut self, ch: char) {
        self.text.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
        self.rewrap();
        self.update_cursor_position();
    }
    
    /// Insert newline at cursor position
    pub fn insert_newline(&mut self) {
        self.text.insert(self.cursor_pos, '\n');
        self.cursor_pos += 1;
        self.rewrap();
        self.update_cursor_position();
    }
    
    /// Delete character before cursor
    pub fn backspace(&mut self) {
        if self.cursor_pos > 0 {
            let mut char_boundary = self.cursor_pos - 1;
            while !self.text.is_char_boundary(char_boundary) && char_boundary > 0 {
                char_boundary -= 1;
            }
            self.text.remove(char_boundary);
            self.cursor_pos = char_boundary;
            self.rewrap();
            self.update_cursor_position();
        }
    }
    
    /// Delete character after cursor
    pub fn delete(&mut self) {
        if self.cursor_pos < self.text.len() {
            let mut next_boundary = self.cursor_pos + 1;
            while !self.text.is_char_boundary(next_boundary) && next_boundary < self.text.len() {
                next_boundary += 1;
            }
            self.text.remove(self.cursor_pos);
            self.rewrap();
            self.update_cursor_position();
        }
    }
    
    /// Move cursor left
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            let mut new_pos = self.cursor_pos - 1;
            while !self.text.is_char_boundary(new_pos) && new_pos > 0 {
                new_pos -= 1;
            }
            self.cursor_pos = new_pos;
            self.update_cursor_position();
        }
    }
    
    /// Move cursor right
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.text.len() {
            let mut new_pos = self.cursor_pos + 1;
            while !self.text.is_char_boundary(new_pos) && new_pos < self.text.len() {
                new_pos += 1;
            }
            self.cursor_pos = new_pos;
            self.update_cursor_position();
        }
    }
    
    /// Move cursor to previous word
    pub fn move_word_left(&mut self) {
        if self.cursor_pos == 0 { return; }
        
        // Skip current whitespace
        while self.cursor_pos > 0 {
            let ch = self.text.chars().nth(self.cursor_pos - 1);
            if ch.map_or(false, |c| !c.is_whitespace()) {
                break;
            }
            self.move_left();
        }
        
        // Move to start of word
        while self.cursor_pos > 0 {
            let ch = self.text.chars().nth(self.cursor_pos - 1);
            if ch.map_or(true, |c| c.is_whitespace()) {
                break;
            }
            self.move_left();
        }
    }
    
    /// Move cursor to next word
    pub fn move_word_right(&mut self) {
        if self.cursor_pos >= self.text.len() { return; }
        
        // Skip current word
        while self.cursor_pos < self.text.len() {
            let ch = self.text.chars().nth(self.cursor_pos);
            if ch.map_or(true, |c| c.is_whitespace()) {
                break;
            }
            self.move_right();
        }
        
        // Skip whitespace
        while self.cursor_pos < self.text.len() {
            let ch = self.text.chars().nth(self.cursor_pos);
            if ch.map_or(false, |c| !c.is_whitespace()) {
                break;
            }
            self.move_right();
        }
    }
    
    /// Move cursor to start of current line
    pub fn move_line_start(&mut self) {
        self.cursor_col = 0;
        self.cursor_pos = self.get_position_from_line_col();
        self.update_cursor_position();
    }
    
    /// Move cursor to end of current line
    pub fn move_line_end(&mut self) {
        if self.cursor_line < self.lines.len() {
            self.cursor_col = self.lines[self.cursor_line].len();
            self.cursor_pos = self.get_position_from_line_col();
            self.update_cursor_position();
        }
    }
    
    /// Move cursor up one line
    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
            self.cursor_pos = self.get_position_from_line_col();
            
            // Adjust scroll if needed
            if self.cursor_line < self.scroll_offset {
                self.scroll_offset = self.cursor_line;
            }
        }
    }
    
    /// Move cursor down one line
    pub fn move_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
            self.cursor_pos = self.get_position_from_line_col();
            
            // Adjust scroll if needed
            if self.cursor_line >= self.scroll_offset + self.max_lines {
                self.scroll_offset = self.cursor_line - self.max_lines + 1;
            }
        }
    }
    
    /// Clear the text field
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.scroll_offset = 0;
    }
    
    /// Rewrap text into lines
    fn rewrap(&mut self) {
        self.lines.clear();
        
        if self.text.is_empty() {
            self.lines.push(String::new());
            return;
        }
        
        let mut current_line = String::new();
        let mut current_width = 0;
        
        for ch in self.text.chars() {
            if ch == '\n' {
                self.lines.push(current_line);
                current_line = String::new();
                current_width = 0;
            } else {
                let char_width = if ch.is_ascii() { 1 } else { 2 }; // Approximate width
                
                if current_width + char_width > self.width && !current_line.is_empty() {
                    // Word wrap
                    let last_space = current_line.rfind(' ');
                    if let Some(space_pos) = last_space {
                        let next_line = current_line[space_pos + 1..].to_string();
                        current_line.truncate(space_pos);
                        self.lines.push(current_line);
                        current_width = next_line.len();
                        current_line = next_line;
                    } else {
                        // No space to break at, force break
                        self.lines.push(current_line);
                        current_line = String::new();
                        current_width = 0;
                    }
                }
                
                current_line.push(ch);
                current_width += char_width;
            }
        }
        
        if !current_line.is_empty() || self.text.ends_with('\n') {
            self.lines.push(current_line);
        }
        
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }
    
    /// Update cursor line and column from text position
    fn update_cursor_position(&mut self) {
        let mut pos = 0;
        for (line_idx, line) in self.lines.iter().enumerate() {
            let line_end = pos + line.len();
            let has_newline = line_idx < self.lines.len() - 1 || self.text.ends_with('\n');
            let actual_line_end = if has_newline { line_end + 1 } else { line_end };
            
            if self.cursor_pos <= actual_line_end {
                self.cursor_line = line_idx;
                self.cursor_col = self.cursor_pos - pos;
                
                // Adjust scroll if cursor moves out of view
                if self.cursor_line < self.scroll_offset {
                    self.scroll_offset = self.cursor_line;
                } else if self.cursor_line >= self.scroll_offset + self.max_lines {
                    self.scroll_offset = self.cursor_line - self.max_lines + 1;
                }
                
                return;
            }
            pos = actual_line_end;
        }
        
        // Fallback to end
        self.cursor_line = self.lines.len() - 1;
        self.cursor_col = self.lines[self.cursor_line].len();
    }
    
    /// Get text position from line and column
    fn get_position_from_line_col(&self) -> usize {
        let mut pos = 0;
        for i in 0..self.cursor_line {
            pos += self.lines[i].len();
            if i < self.lines.len() - 1 || self.text.ends_with('\n') {
                pos += 1; // Newline
            }
        }
        pos + self.cursor_col.min(self.lines.get(self.cursor_line).map_or(0, |l| l.len()))
    }
    
    /// Get visible lines
    pub fn visible_lines(&self) -> &[String] {
        let end = (self.scroll_offset + self.max_lines).min(self.lines.len());
        &self.lines[self.scroll_offset..end]
    }
    
    /// Get the height needed for current text
    pub fn get_height(&self) -> usize {
        self.lines.len().min(self.max_lines)
    }
}