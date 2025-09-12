//! Improved input handling with Mac shortcuts and multiline support

use std::io::{self, Read};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};
use std::thread;
use std::os::unix::io::AsRawFd;

#[derive(Debug, Clone)]
pub enum InputEvent {
    Char(char),
    Backspace,
    Delete,
    Enter,
    NewLine, // Shift+Enter
    Escape,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    WordLeft,  // Option+Left on Mac
    WordRight, // Option+Right on Mac
    LineStart, // Cmd+Left on Mac
    LineEnd,   // Cmd+Right on Mac
}

pub struct InputHandler {
    receiver: Receiver<InputEvent>,
    _thread: thread::JoinHandle<()>,
    original_termios: libc::termios,
}

impl InputHandler {
    pub fn new() -> io::Result<Self> {
        let stdin_fd = io::stdin().as_raw_fd();
        
        // Save original terminal settings
        let mut original_termios = unsafe { std::mem::zeroed() };
        unsafe { libc::tcgetattr(stdin_fd, &mut original_termios) };
        
        // Enable raw mode
        unsafe {
            let mut raw = original_termios;
            libc::cfmakeraw(&mut raw);
            // Keep ISIG to handle Ctrl+C
            raw.c_lflag |= libc::ISIG;
            libc::tcsetattr(stdin_fd, libc::TCSANOW, &raw);
        }
        
        let (sender, receiver) = channel();
        
        // Spawn input thread
        let handle = thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = [0u8; 16]; // Larger buffer for escape sequences
            
            loop {
                match stdin.read(&mut buffer) {
                    Ok(n) if n > 0 => {
                        Self::process_input(&buffer[..n], &sender);
                    }
                    _ => break,
                }
            }
        });
        
        Ok(InputHandler {
            receiver,
            _thread: handle,
            original_termios,
        })
    }
    
    fn process_input(bytes: &[u8], sender: &Sender<InputEvent>) {
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                // Control characters
                0x1B => {
                    // ESC sequence
                    if i + 1 < bytes.len() {
                        match bytes[i + 1] {
                            b'[' if i + 2 < bytes.len() => {
                                // ANSI escape sequence
                                match bytes[i + 2] {
                                    b'A' => { sender.send(InputEvent::Up).ok(); i += 2; }
                                    b'B' => { sender.send(InputEvent::Down).ok(); i += 2; }
                                    b'C' => { sender.send(InputEvent::Right).ok(); i += 2; }
                                    b'D' => { sender.send(InputEvent::Left).ok(); i += 2; }
                                    b'H' => { sender.send(InputEvent::Home).ok(); i += 2; }
                                    b'F' => { sender.send(InputEvent::End).ok(); i += 2; }
                                    b'3' if i + 3 < bytes.len() && bytes[i + 3] == b'~' => {
                                        sender.send(InputEvent::Delete).ok(); i += 3;
                                    }
                                    b'1' if i + 3 < bytes.len() && bytes[i + 3] == b';' => {
                                        // Modified arrow keys (Shift/Ctrl/Cmd)
                                        if i + 5 < bytes.len() {
                                            match (bytes[i + 4], bytes[i + 5]) {
                                                (b'5', b'D') => { sender.send(InputEvent::WordLeft).ok(); i += 5; }
                                                (b'5', b'C') => { sender.send(InputEvent::WordRight).ok(); i += 5; }
                                                (b'9', b'D') => { sender.send(InputEvent::LineStart).ok(); i += 5; }
                                                (b'9', b'C') => { sender.send(InputEvent::LineEnd).ok(); i += 5; }
                                                _ => i += 5,
                                            }
                                        }
                                    }
                                    _ => i += 2,
                                }
                            }
                            b'b' => {
                                // Option+Left (word backward)
                                sender.send(InputEvent::WordLeft).ok();
                                i += 1;
                            }
                            b'f' => {
                                // Option+Right (word forward)
                                sender.send(InputEvent::WordRight).ok();
                                i += 1;
                            }
                            _ => {
                                // Just ESC
                                sender.send(InputEvent::Escape).ok();
                            }
                        }
                    } else {
                        sender.send(InputEvent::Escape).ok();
                    }
                }
                0x7F | 0x08 => {
                    // Backspace (DEL or BS)
                    sender.send(InputEvent::Backspace).ok();
                }
                0x0D => {
                    // Enter (CR)
                    sender.send(InputEvent::Enter).ok();
                }
                0x0A => {
                    // Line Feed (Shift+Enter on some terminals)
                    sender.send(InputEvent::NewLine).ok();
                }
                0x01 => {
                    // Ctrl+A (beginning of line)
                    sender.send(InputEvent::LineStart).ok();
                }
                0x05 => {
                    // Ctrl+E (end of line)
                    sender.send(InputEvent::LineEnd).ok();
                }
                b if b >= 0x20 && b < 0x7F => {
                    // Regular ASCII character
                    sender.send(InputEvent::Char(b as char)).ok();
                }
                b if b >= 0xC0 => {
                    // UTF-8 multibyte character
                    let char_bytes = Self::extract_utf8_char(bytes, i);
                    if let Ok(s) = std::str::from_utf8(&char_bytes) {
                        if let Some(ch) = s.chars().next() {
                            sender.send(InputEvent::Char(ch)).ok();
                            i += char_bytes.len() - 1;
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    
    fn extract_utf8_char(bytes: &[u8], start: usize) -> Vec<u8> {
        let first = bytes[start];
        let len = if first & 0x80 == 0 { 1 }
        else if first & 0xE0 == 0xC0 { 2 }
        else if first & 0xF0 == 0xE0 { 3 }
        else if first & 0xF8 == 0xF0 { 4 }
        else { 1 };
        
        bytes[start..std::cmp::min(start + len, bytes.len())].to_vec()
    }
    
    pub fn poll(&self) -> Option<InputEvent> {
        match self.receiver.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}

impl Drop for InputHandler {
    fn drop(&mut self) {
        // Restore terminal settings
        unsafe {
            libc::tcsetattr(io::stdin().as_raw_fd(), libc::TCSANOW, &self.original_termios);
        }
        // Show cursor
        print!("\x1b[?25h");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
}