use unicode_segmentation::UnicodeSegmentation;
use winit::event::ModifiersState;

pub struct InputField {
    text: String,
    cursor_pos: usize,
    lines: Vec<String>,
    cursor_line: usize,
    cursor_col: usize,
    pub modifiers: ModifiersState,
    selection: Option<(usize, usize)>,
}

impl InputField {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            lines: vec![String::new()],
            cursor_line: 0,
            cursor_col: 0,
            modifiers: ModifiersState::empty(),
            selection: None,
        }
    }
    
    pub fn insert_char(&mut self, ch: char) {
        self.clear_selection();
        self.text.insert(self.cursor_pos, ch);
        self.cursor_pos += ch.len_utf8();
        self.update_lines();
        self.update_cursor_position();
    }
    
    pub fn insert_newline(&mut self) {
        self.clear_selection();
        self.text.insert(self.cursor_pos, '\n');
        self.cursor_pos += 1;
        self.update_lines();
        self.cursor_line += 1;
        self.cursor_col = 0;
    }
    
    pub fn backspace(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            return;
        }
        
        if self.cursor_pos > 0 {
            let prev_char_boundary = self.prev_char_boundary(self.cursor_pos);
            self.text.drain(prev_char_boundary..self.cursor_pos);
            self.cursor_pos = prev_char_boundary;
            self.update_lines();
            self.update_cursor_position();
        }
    }
    
    pub fn delete(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            return;
        }
        
        if self.cursor_pos < self.text.len() {
            let next_char_boundary = self.next_char_boundary(self.cursor_pos);
            self.text.drain(self.cursor_pos..next_char_boundary);
            self.update_lines();
        }
    }
    
    pub fn delete_word_backward(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        
        let word_start = self.find_word_boundary_left(self.cursor_pos);
        self.text.drain(word_start..self.cursor_pos);
        self.cursor_pos = word_start;
        self.update_lines();
        self.update_cursor_position();
    }
    
    pub fn delete_word_forward(&mut self) {
        if self.cursor_pos >= self.text.len() {
            return;
        }
        
        let word_end = self.find_word_boundary_right(self.cursor_pos);
        self.text.drain(self.cursor_pos..word_end);
        self.update_lines();
    }
    
    pub fn move_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos = self.prev_char_boundary(self.cursor_pos);
            self.update_cursor_position();
        }
    }
    
    pub fn move_right(&mut self) {
        if self.cursor_pos < self.text.len() {
            self.cursor_pos = self.next_char_boundary(self.cursor_pos);
            self.update_cursor_position();
        }
    }
    
    pub fn move_word_left(&mut self) {
        self.cursor_pos = self.find_word_boundary_left(self.cursor_pos);
        self.update_cursor_position();
    }
    
    pub fn move_word_right(&mut self) {
        self.cursor_pos = self.find_word_boundary_right(self.cursor_pos);
        self.update_cursor_position();
    }
    
    pub fn move_to_line_start(&mut self) {
        if self.cursor_line < self.lines.len() {
            let mut pos = 0;
            for i in 0..self.cursor_line {
                pos += self.lines[i].len() + 1; // +1 for newline
            }
            self.cursor_pos = pos;
            self.cursor_col = 0;
        }
    }
    
    pub fn move_to_line_end(&mut self) {
        if self.cursor_line < self.lines.len() {
            let mut pos = 0;
            for i in 0..=self.cursor_line {
                if i == self.cursor_line {
                    pos += self.lines[i].len();
                } else {
                    pos += self.lines[i].len() + 1; // +1 for newline
                }
            }
            self.cursor_pos = pos;
            self.cursor_col = self.lines[self.cursor_line].len();
        }
    }
    
    pub fn move_up(&mut self) {
        if self.cursor_line > 0 {
            self.cursor_line -= 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
            self.recalculate_cursor_pos();
        }
    }
    
    pub fn move_down(&mut self) {
        if self.cursor_line < self.lines.len() - 1 {
            self.cursor_line += 1;
            self.cursor_col = self.cursor_col.min(self.lines[self.cursor_line].len());
            self.recalculate_cursor_pos();
        }
    }
    
    pub fn select_all(&mut self) {
        self.selection = Some((0, self.text.len()));
        self.cursor_pos = self.text.len();
        self.update_cursor_position();
    }
    
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.lines = vec![String::new()];
        self.cursor_line = 0;
        self.cursor_col = 0;
        self.selection = None;
    }
    
    pub fn get_text(&self) -> String {
        self.text.clone()
    }
    
    pub fn get_lines(&self) -> &[String] {
        &self.lines
    }
    
    pub fn get_height(&self) -> f32 {
        let line_height = 20.0;
        let padding = 20.0;
        let min_height = 50.0;
        let max_height = 150.0; // Max 5 lines
        
        let height = (self.lines.len() as f32 * line_height) + padding;
        height.max(min_height).min(max_height)
    }
    
    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.cursor_col, self.cursor_line)
    }
    
    // Helper methods
    
    fn update_lines(&mut self) {
        self.lines = self.text.split('\n').map(|s| s.to_string()).collect();
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
    }
    
    fn update_cursor_position(&mut self) {
        let mut pos = 0;
        self.cursor_line = 0;
        self.cursor_col = 0;
        
        for (line_idx, line) in self.lines.iter().enumerate() {
            let line_start = pos;
            let line_end = pos + line.len();
            
            if self.cursor_pos <= line_end {
                self.cursor_line = line_idx;
                self.cursor_col = self.cursor_pos - line_start;
                break;
            }
            
            pos = line_end + 1; // +1 for newline
        }
    }
    
    fn recalculate_cursor_pos(&mut self) {
        self.cursor_pos = 0;
        for i in 0..self.cursor_line {
            self.cursor_pos += self.lines[i].len() + 1; // +1 for newline
        }
        self.cursor_pos += self.cursor_col;
    }
    
    fn prev_char_boundary(&self, pos: usize) -> usize {
        let mut graphemes = self.text[..pos].grapheme_indices(true);
        graphemes.next_back().map(|(i, _)| i).unwrap_or(0)
    }
    
    fn next_char_boundary(&self, pos: usize) -> usize {
        let mut graphemes = self.text[pos..].grapheme_indices(true);
        graphemes.nth(1).map(|(i, _)| pos + i).unwrap_or(self.text.len())
    }
    
    fn find_word_boundary_left(&self, pos: usize) -> usize {
        let text_before = &self.text[..pos];
        let mut last_word_start = 0;
        
        for (i, ch) in text_before.char_indices().rev() {
            if ch.is_whitespace() {
                if i < pos - 1 {
                    return i + ch.len_utf8();
                }
            } else if last_word_start == 0 {
                last_word_start = i;
            }
        }
        
        0
    }
    
    fn find_word_boundary_right(&self, pos: usize) -> usize {
        let text_after = &self.text[pos..];
        let mut found_non_space = false;
        
        for (i, ch) in text_after.char_indices() {
            if ch.is_whitespace() {
                if found_non_space {
                    return pos + i;
                }
            } else {
                found_non_space = true;
            }
        }
        
        self.text.len()
    }
    
    fn clear_selection(&mut self) {
        self.selection = None;
    }
    
    fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection {
            self.text.drain(start..end);
            self.cursor_pos = start;
            self.selection = None;
            self.update_lines();
            self.update_cursor_position();
        }
    }
}