use std::io::{self, Read, Write, stdout};
use std::os::unix::io::AsRawFd;

fn main() -> io::Result<()> {
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
    
    println!("Testing raw input. Press keys to see what's received. ESC to exit.");
    println!("Try: regular chars, arrows, Option+arrows, Cmd+arrows\n");
    
    let mut stdin = io::stdin();
    let mut buffer = [0u8; 16];
    
    loop {
        match stdin.read(&mut buffer) {
            Ok(n) if n > 0 => {
                print!("Received {} bytes: ", n);
                for i in 0..n {
                    print!("{:02X} ", buffer[i]);
                }
                print!(" | ");
                for i in 0..n {
                    let b = buffer[i];
                    if b >= 0x20 && b < 0x7F {
                        print!("{}", b as char);
                    } else {
                        print!(".");
                    }
                }
                println!();
                stdout().flush()?;
                
                // Exit on ESC
                if buffer[0] == 0x1B && n == 1 {
                    break;
                }
            }
            _ => {}
        }
    }
    
    // Restore terminal
    unsafe {
        libc::tcsetattr(stdin_fd, libc::TCSANOW, &original_termios);
    }
    
    Ok(())
}