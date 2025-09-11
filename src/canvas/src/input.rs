// Terminal input handling for Canvas
use std::io::{self, Read};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread;
use std::os::unix::io::AsRawFd;

pub enum InputEvent {
    Char(char),
    Backspace,
    Enter,
    Escape,
}

pub struct InputHandler {
    receiver: Receiver<InputEvent>,
    _thread: thread::JoinHandle<()>,
}

impl InputHandler {
    pub fn new() -> io::Result<Self> {
        // Set terminal to raw mode
        let stdin_fd = io::stdin().as_raw_fd();
        
        // Save current terminal settings
        let mut termios = unsafe { std::mem::zeroed() };
        unsafe { libc::tcgetattr(stdin_fd, &mut termios) };
        
        // Enable raw mode
        unsafe {
            let mut raw = termios;
            libc::cfmakeraw(&mut raw);
            libc::tcsetattr(stdin_fd, libc::TCSANOW, &raw);
        }
        
        // Hide cursor
        print!("\x1b[?25l");
        
        let (sender, receiver) = channel();
        
        // Spawn input thread
        let handle = thread::spawn(move || {
            let mut stdin = io::stdin();
            let mut buffer = [0u8; 1];
            
            loop {
                if stdin.read_exact(&mut buffer).is_ok() {
                    let byte = buffer[0];
                    
                    let event = match byte {
                        0x1B => InputEvent::Escape, // ESC
                        0x7F | 0x08 => InputEvent::Backspace, // Backspace/Delete
                        0x0D => InputEvent::Enter, // Enter
                        b if b >= 0x20 && b < 0x7F => InputEvent::Char(byte as char),
                        _ => continue, // Ignore other control characters
                    };
                    
                    if sender.send(event).is_err() {
                        break; // Receiver dropped
                    }
                    
                    // Exit on ESC
                    if matches!(byte, 0x1B) {
                        break;
                    }
                }
            }
            
            // Restore terminal
            unsafe {
                libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios);
            }
            // Show cursor
            print!("\x1b[?25h");
        });
        
        Ok(InputHandler {
            receiver,
            _thread: handle,
        })
    }
    
    pub fn poll(&self) -> Option<InputEvent> {
        self.receiver.try_recv().ok()
    }
}

impl Drop for InputHandler {
    fn drop(&mut self) {
        // Restore terminal on drop
        let stdin_fd = io::stdin().as_raw_fd();
        let mut termios = unsafe { std::mem::zeroed() };
        unsafe { 
            libc::tcgetattr(stdin_fd, &mut termios);
            termios.c_lflag |= libc::ICANON | libc::ECHO;
            libc::tcsetattr(stdin_fd, libc::TCSANOW, &termios);
        }
        // Show cursor
        print!("\x1b[?25h");
    }
}