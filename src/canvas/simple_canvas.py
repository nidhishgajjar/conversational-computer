#!/usr/bin/env python3
"""
Simple Canvas - Minimal working compositor
Using direct framebuffer approach for simplicity
"""

import os
import sys
import termios
import tty
import select
import time
from dataclasses import dataclass
from typing import List, Optional

@dataclass
class Block:
    """A conversation block"""
    sender: str
    message: str
    timestamp: float

class SimpleCanvas:
    """
    Simplified Canvas using terminal UI first
    This proves the concept before we tackle Wayland
    """
    
    def __init__(self):
        self.running = False
        self.blocks: List[Block] = []
        self.input_buffer = ""
        self.cursor_pos = 0
        
        # Save terminal settings
        self.old_settings = None
        
    def setup_terminal(self):
        """Setup terminal for raw input"""
        self.old_settings = termios.tcgetattr(sys.stdin)
        tty.setraw(sys.stdin)
        
        # Clear screen and hide cursor
        sys.stdout.write('\033[2J\033[H\033[?25l')
        sys.stdout.flush()
        
    def restore_terminal(self):
        """Restore terminal settings"""
        if self.old_settings:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, self.old_settings)
        
        # Show cursor and clear screen
        sys.stdout.write('\033[?25h\033[2J\033[H')
        sys.stdout.flush()
        
    def render(self):
        """Render the CUI"""
        # Clear screen
        sys.stdout.write('\033[2J\033[H')
        
        # Get terminal size
        rows, cols = os.popen('stty size', 'r').read().split()
        rows, cols = int(rows), int(cols)
        
        # Title bar
        title = " CANVAS - Conversational Computer "
        padding = (cols - len(title)) // 2
        sys.stdout.write('\033[48;5;236m' + ' ' * cols + '\033[0m\n')
        sys.stdout.write('\033[48;5;236m' + ' ' * padding + '\033[1;37m' + title + '\033[0m')
        sys.stdout.write('\033[48;5;236m' + ' ' * (cols - padding - len(title)) + '\033[0m\n')
        sys.stdout.write('\033[48;5;236m' + ' ' * cols + '\033[0m\n')
        
        # Conversation area
        current_row = 4
        for block in self.blocks[-10:]:  # Show last 10 blocks
            sys.stdout.write(f'\033[{current_row};1H')
            sys.stdout.write(f'\033[1;36m{block.sender}:\033[0m {block.message[:cols-10]}\n')
            current_row += 2
            if current_row > rows - 6:
                break
        
        # Input bar at bottom
        input_row = rows - 3
        sys.stdout.write(f'\033[{input_row};1H')
        sys.stdout.write('\033[48;5;238m' + ' ' * cols + '\033[0m')
        
        # Input area with border
        input_start = (cols - 60) // 2 if cols > 60 else 5
        sys.stdout.write(f'\033[{input_row + 1};{input_start}H')
        sys.stdout.write('\033[48;5;235m[SPOC>] ')
        sys.stdout.write('\033[1;37m' + self.input_buffer + '\033[0m')
        sys.stdout.write('\033[48;5;235m' + ' ' * (50 - len(self.input_buffer)) + '\033[0m')
        
        # Status line
        sys.stdout.write(f'\033[{rows};1H')
        status = " Alt+Q: Quit | Enter: Send | Shift+Enter: Newline "
        sys.stdout.write('\033[48;5;234m' + status + ' ' * (cols - len(status)) + '\033[0m')
        
        # Position cursor
        cursor_col = input_start + 8 + self.cursor_pos
        sys.stdout.write(f'\033[{input_row + 1};{cursor_col}H')
        sys.stdout.write('\033[?25h')  # Show cursor
        
        sys.stdout.flush()
        
    def handle_input(self):
        """Handle keyboard input"""
        if select.select([sys.stdin], [], [], 0)[0]:
            char = sys.stdin.read(1)
            
            # Check for special keys
            if ord(char) == 27:  # ESC sequence
                next1 = sys.stdin.read(1)
                if next1 == 'q':  # Alt+Q
                    self.running = False
                    return
                    
            elif ord(char) == 13:  # Enter
                if self.input_buffer:
                    # Add to conversation
                    self.blocks.append(Block("You", self.input_buffer, time.time()))
                    
                    # Simulate SPOC response
                    response = f"Processing: '{self.input_buffer}'"
                    self.blocks.append(Block("SPOC", response, time.time()))
                    
                    # Clear input
                    self.input_buffer = ""
                    self.cursor_pos = 0
                    
            elif ord(char) == 127:  # Backspace
                if self.cursor_pos > 0:
                    self.input_buffer = self.input_buffer[:self.cursor_pos-1] + self.input_buffer[self.cursor_pos:]
                    self.cursor_pos -= 1
                    
            elif 32 <= ord(char) <= 126:  # Printable characters
                self.input_buffer = self.input_buffer[:self.cursor_pos] + char + self.input_buffer[self.cursor_pos:]
                self.cursor_pos += 1
                
    def run(self):
        """Main loop"""
        try:
            self.setup_terminal()
            self.running = True
            
            # Welcome message
            self.blocks.append(Block("CANVAS", "Conversational Computer initialized", time.time()))
            self.blocks.append(Block("CANVAS", "Type your message and press Enter", time.time()))
            
            while self.running:
                self.render()
                self.handle_input()
                time.sleep(0.01)  # Small delay to reduce CPU usage
                
        except KeyboardInterrupt:
            pass
        finally:
            self.restore_terminal()
            print("\nCanvas terminated.")


def main():
    """Entry point"""
    print("Starting Simple Canvas...")
    print("This is a terminal prototype of the Conversational Computer")
    print("Press any key to continue...")
    input()
    
    canvas = SimpleCanvas()
    canvas.run()
    return 0


if __name__ == "__main__":
    sys.exit(main())