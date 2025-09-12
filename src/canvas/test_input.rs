use std::io::{self, stdin, stdout, Write};
use std::thread;
use std::time::Duration;

mod improved_input;
use improved_input::{InputHandler, InputEvent};

fn main() -> io::Result<()> {
    println!("Testing improved input handler...");
    println!("Try typing, using arrow keys, Cmd+arrows, Option+arrows");
    println!("Press ESC to exit\n");
    
    let input_handler = InputHandler::new()?;
    let mut text = String::new();
    let mut cursor_pos = 0;
    
    loop {
        // Clear line and redraw
        print!("\r\x1b[K");
        print!("Input: {}", text);
        
        // Position cursor
        print!("\r\x1b[{}C", 7 + cursor_pos);
        stdout().flush()?;
        
        if let Some(event) = input_handler.poll() {
            match event {
                InputEvent::Char(ch) => {
                    text.insert(cursor_pos, ch);
                    cursor_pos += 1;
                    println!("\nReceived char: {:?}", ch);
                }
                InputEvent::Backspace => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        text.remove(cursor_pos);
                        println!("\nBackspace pressed");
                    }
                }
                InputEvent::Left => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                        println!("\nLeft arrow");
                    }
                }
                InputEvent::Right => {
                    if cursor_pos < text.len() {
                        cursor_pos += 1;
                        println!("\nRight arrow");
                    }
                }
                InputEvent::WordLeft => {
                    println!("\nWord left (Option+Left)");
                }
                InputEvent::WordRight => {
                    println!("\nWord right (Option+Right)");
                }
                InputEvent::LineStart => {
                    cursor_pos = 0;
                    println!("\nLine start (Cmd+Left)");
                }
                InputEvent::LineEnd => {
                    cursor_pos = text.len();
                    println!("\nLine end (Cmd+Right)");
                }
                InputEvent::Escape => {
                    println!("\nESC pressed, exiting...");
                    break;
                }
                _ => {
                    println!("\nOther event: {:?}", event);
                }
            }
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    Ok(())
}