# Canvas Compositor Implementation Guide

## Overview
Canvas is a minimal Wayland compositor designed specifically for the Conversational Computer. Unlike traditional compositors, Canvas doesn't manage windows - it provides a single surface for SPOC to conduct conversations.

## Technical Stack

### Core Dependencies
```bash
# Arch packages needed
pacman -S wayland wayland-protocols wlroots libinput mesa libgl \
          cairo pango freetype2 libdrm libevdev pixman libxkbcommon
```

### Python Implementation (Phase 1)
```python
# Key imports
from pywayland.server import Display
from wlroots.backend import Backend
from wlroots.renderer import Renderer
from wlroots.wlr_types import Scene, Compositor, Seat
```

## Basic Compositor Structure

```python
class CanvasCompositor:
    def __init__(self):
        # Wayland display server
        self.display = Display()
        
        # Backend (DRM/KMS for real hardware)
        self.backend = Backend(self.display)
        
        # Renderer (OpenGL ES)
        self.renderer = self.backend.renderer
        
        # Scene graph for rendering
        self.scene = Scene()
        
        # Input handling
        self.seat = Seat(self.display, "seat0")
        
        # SPOC components
        self.input_bar = None
        self.conversation_area = None
        
    def render_frame(self):
        # Clear to background color
        self.renderer.clear([0.1, 0.12, 0.15, 1.0])
        
        # Render conversation blocks
        self.render_blocks()
        
        # Render input bar
        self.render_input_bar()
        
    def run(self):
        # Start backend
        self.backend.start()
        
        # Create socket
        socket = self.display.add_socket()
        
        # Run event loop
        self.display.run()
```

## Running Without X/Wayland

### TTY Setup
```bash
# Switch to TTY1 (Ctrl+Alt+F1)
# Login as root or user with video group

# Set environment
export WLR_BACKENDS=drm
export WLR_LIBINPUT_NO_DEVICES=1  # If no input devices

# Run compositor
python3 /opt/canvas/canvas.py
```

### Systemd Service
```ini
[Unit]
Description=Canvas Compositor
After=systemd-user-sessions.service

[Service]
Type=simple
ExecStart=/usr/bin/python3 /opt/canvas/canvas.py
Restart=on-failure
User=root
PAMName=login
TTYPath=/dev/tty1
StandardInput=tty
StandardOutput=journal

[Install]
WantedBy=graphical.target
```

## Text Rendering

### Using Cairo/Pango
```python
import cairo
import gi
gi.require_version('Pango', '1.0')
from gi.repository import Pango, PangoCairo

def render_text(self, text, x, y):
    # Create Cairo surface
    surface = cairo.ImageSurface(cairo.FORMAT_ARGB32, width, height)
    ctx = cairo.Context(surface)
    
    # Setup Pango
    layout = PangoCairo.create_layout(ctx)
    layout.set_text(text, -1)
    
    # Set font
    font = Pango.FontDescription("monospace 14")
    layout.set_font_description(font)
    
    # Render
    ctx.move_to(x, y)
    PangoCairo.show_layout(ctx, layout)
    
    # Convert to texture for GPU
    return self.renderer.texture_from_pixels(surface.get_data())
```

## Input Handling

### Keyboard Events
```python
def on_keyboard_key(self, keyboard, event):
    keycode = event.keycode
    state = event.state  # 1 = pressed, 0 = released
    
    if state == 1:
        # Get keysym
        sym = keyboard.xkb_state.key_get_one_sym(keycode + 8)
        
        # Handle text input
        if self.input_bar.has_focus():
            if sym == XKB_KEY_Return:
                if not shift_pressed:
                    self.submit_input()
                else:
                    self.input_bar.add_newline()
            else:
                # Add character to input
                char = xkb.keysym_to_utf8(sym)
                self.input_bar.add_text(char)
```

## Memory-Mapped Framebuffer (Alternative)

For even simpler implementation without wlroots:

```python
import mmap
import struct

class FramebufferCanvas:
    def __init__(self):
        # Open framebuffer device
        self.fb = open('/dev/fb0', 'r+b')
        
        # Get screen info via ioctl
        # ... 
        
        # Memory map
        self.screen = mmap.mmap(self.fb.fileno(), 
                                width * height * 4)
        
    def set_pixel(self, x, y, r, g, b):
        offset = (y * self.width + x) * 4
        self.screen[offset:offset+4] = struct.pack('BBBB', b, g, r, 0)
        
    def clear(self, color):
        for y in range(self.height):
            for x in range(self.width):
                self.set_pixel(x, y, *color)
```

## Development Workflow

### Testing Nested
```bash
# Install cage for testing
pacman -S cage

# Run Canvas inside cage
cage python3 /opt/canvas/canvas.py
```

### Direct TTY Development
```bash
# TTY1 for Canvas
# TTY2 for code editing (vim)
# TTY3 for logs/debugging

# Switch with Ctrl+Alt+F1/F2/F3
```

## Performance Considerations

1. **Single Buffer** - No need for double buffering initially
2. **Damage Tracking** - Only redraw changed regions
3. **Text Caching** - Cache rendered text as textures
4. **GPU Acceleration** - Use OpenGL ES for all rendering

## Debugging

### Enable Logging
```bash
export WAYLAND_DEBUG=1
export WLR_LOG_LEVEL=debug
```

### Check DRM/KMS
```bash
# List DRM devices
ls /dev/dri/

# Check KMS capabilities
drmModeGetResources
```

## Migration Path to Rust

### Rust Dependencies
```toml
[dependencies]
smithay = "0.3"  # Wayland compositor framework
winit = "0.29"   # Window/input handling
wgpu = "0.18"    # GPU rendering
```

### Rust Structure
```rust
struct Canvas {
    display: Display<State>,
    backend: Backend,
    renderer: Renderer,
    spoc: Spoc,
}

impl Canvas {
    fn new() -> Self {
        // Initialize
    }
    
    fn run(&mut self) {
        // Event loop
    }
}
```

## Current Status

- [x] Basic compositor structure defined
- [x] Rendering pipeline planned
- [x] Input handling designed
- [ ] Text rendering implementation
- [ ] SPOC integration
- [ ] Block system
- [ ] Testing on real hardware

## Next Implementation Steps

1. Get basic compositor running on TTY
2. Render solid background
3. Add text rendering
4. Implement input bar
5. Add conversation blocks
6. Connect to AI backend