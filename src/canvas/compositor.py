#!/usr/bin/env python3
"""
Canvas - Minimal Wayland Compositor for Conversational Computer
Phase 1: Basic compositor that renders background and handles input
"""

import os
import sys
import time
import logging
from typing import Optional

# Setup logging
logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger("Canvas")

try:
    from pywayland.server import Display
    from wlroots.backend import Backend
    from wlroots.renderer import Renderer
    from wlroots.allocator import Allocator
    from wlroots.wlr_types.compositor import Compositor
    from wlroots.wlr_types.output_layout import OutputLayout
    from wlroots.wlr_types.scene import Scene, SceneOutput
    from wlroots.wlr_types.seat import Seat
    from wlroots.wlr_types.data_device_manager import DataDeviceManager
    from wlroots.wlr_types.input_device import InputDevice, InputDeviceType
    from wlroots.wlr_types.keyboard import Keyboard
    from wlroots.wlr_types.output import Output
    from xkbcommon import xkb
except ImportError as e:
    logger.error(f"Missing dependency: {e}")
    logger.error("Install: python-pywayland python-xkbcommon wlroots")
    sys.exit(1)


class Canvas:
    """
    Canvas compositor - the visual foundation for the Conversational Computer
    """
    
    def __init__(self):
        logger.info("Initializing Canvas compositor...")
        
        # Core Wayland components
        self.display = Display()
        self.backend = None
        self.renderer = None
        self.allocator = None
        self.compositor = None
        
        # Scene graph for rendering
        self.scene = None
        self.output_layout = None
        
        # Input handling
        self.seat = None
        self.keyboards = []
        
        # Canvas state
        self.running = False
        self.outputs = []
        
        # Visual settings
        self.bg_color = (0.08, 0.10, 0.14, 1.0)  # Dark blue-gray
        
    def init(self) -> bool:
        """Initialize all compositor components"""
        try:
            # Create backend
            logger.info("Creating backend...")
            self.backend = Backend(self.display)
            
            # Create renderer (auto-detect best available)
            from wlroots.renderer import Renderer
            self.renderer = Renderer.autocreate(self.backend)
            self.allocator = Allocator.autocreate(self.backend, self.renderer)
            
            # Initialize renderer with display
            self.renderer.init_display(self.display)
            
            # Create compositor global (version 5 for latest protocol)
            self.compositor = Compositor(self.display, 5, self.renderer)
            
            # Create output layout (no display argument)
            self.output_layout = OutputLayout()
            
            # Create scene
            self.scene = Scene()
            self.scene.attach_output_layout(self.output_layout)
            
            # Setup seat for input
            self.seat = Seat(self.display, "seat0")
            
            # Data device manager for clipboard
            DataDeviceManager(self.display)
            
            # Connect backend signals
            self.backend.new_output_event.add(self.handle_new_output)
            self.backend.new_input_event.add(self.handle_new_input)
            
            logger.info("Canvas initialization complete")
            return True
            
        except Exception as e:
            logger.error(f"Failed to initialize: {e}")
            return False
    
    def handle_new_output(self, output: Output):
        """Handle new display/monitor connection"""
        logger.info(f"New output detected: {output.name}")
        
        # Choose preferred mode or fallback
        if output.preferred_mode:
            output.set_mode(output.preferred_mode)
        elif output.modes:
            output.set_mode(output.modes[0])
        
        # Initialize rendering
        output.init_render(self.allocator, self.renderer)
        output.enable()
        
        # Add to layout
        self.output_layout.add_auto(output)
        
        # Create scene output
        scene_output = SceneOutput(self.scene, output)
        scene_output.set_position(0, 0)
        
        # Connect frame event
        output.frame_event.add(self.handle_frame)
        
        # Commit changes
        output.commit()
        self.outputs.append(output)
        
        logger.info(f"Output {output.name} configured: {output.width}x{output.height}")
    
    def handle_frame(self, output: Output):
        """Render a frame"""
        # Get scene output
        scene_output = self.scene.get_scene_output(output)
        if not scene_output:
            return
        
        # Start rendering
        with output:
            # Clear to background color
            self.renderer.begin(output.width, output.height)
            self.renderer.clear(self.bg_color)
            
            # TODO: Render SPOC input bar here
            self.render_spoc_ui(output)
            
            # Render scene
            self.scene.render_output(output, 0, 0, None)
            
            # Finish frame
            self.renderer.end()
        
        # Present frame
        output.commit()
        
        # Schedule next frame if needed
        # output.schedule_frame()
    
    def render_spoc_ui(self, output: Output):
        """Render SPOC interface elements"""
        # TODO: Implement input bar rendering
        # For now, just a placeholder
        pass
    
    def handle_new_input(self, device: InputDevice):
        """Handle new input device"""
        if device.device_type == InputDeviceType.KEYBOARD:
            self.setup_keyboard(device)
        elif device.device_type == InputDeviceType.POINTER:
            logger.info(f"Mouse connected: {device.name}")
            # TODO: Handle mouse
    
    def setup_keyboard(self, device: InputDevice):
        """Configure keyboard"""
        logger.info(f"Setting up keyboard: {device.name}")
        
        keyboard = Keyboard(device.keyboard)
        
        # Setup XKB keymap
        context = xkb.Context()
        keymap = context.keymap_new_from_names()
        keyboard.set_keymap(keymap)
        keyboard.set_repeat_info(25, 600)  # rate, delay
        
        # Connect events
        keyboard.modifiers_event.add(self.handle_keyboard_modifiers)
        keyboard.key_event.add(self.handle_keyboard_key)
        
        # Set as seat keyboard
        self.seat.set_keyboard(keyboard.base)
        self.keyboards.append(keyboard)
    
    def handle_keyboard_modifiers(self, keyboard: Keyboard):
        """Handle modifier keys"""
        self.seat.keyboard_notify_modifiers(keyboard.modifiers)
    
    def handle_keyboard_key(self, keyboard: Keyboard, event):
        """Handle key press/release"""
        if event.state == 1:  # Key press
            keycode = event.keycode + 8
            keysyms = keyboard.xkb_state.key_get_syms(keycode)
            
            for sym in keysyms:
                # Check for exit combo (Alt+Escape)
                if sym == xkb.keysym_from_name("Escape"):
                    if keyboard.xkb_state.mod_name_is_active(
                        xkb.MOD_NAME_ALT,
                        xkb.STATE_MODS_EFFECTIVE
                    ):
                        logger.info("Exit combo pressed (Alt+Escape)")
                        self.stop()
                        return
                
                # TODO: Handle SPOC input
                logger.debug(f"Key pressed: {xkb.keysym_get_name(sym)}")
        
        # Notify seat
        self.seat.keyboard_notify_key(
            event.time_msec,
            event.keycode, 
            event.state
        )
    
    def run(self) -> int:
        """Main compositor loop"""
        # Initialize
        if not self.init():
            return 1
        
        # Start backend
        if not self.backend.start():
            logger.error("Failed to start backend")
            logger.error("Make sure you're running from TTY or with proper permissions")
            return 1
        
        # Create Wayland socket
        socket = self.display.add_socket()
        if not socket:
            logger.error("Failed to create Wayland socket")
            return 1
        
        socket_name = socket.decode() if isinstance(socket, bytes) else socket
        logger.info(f"Canvas compositor running on WAYLAND_DISPLAY={socket_name}")
        logger.info("Press Alt+Escape to exit")
        
        # Set environment
        os.environ["WAYLAND_DISPLAY"] = socket_name
        
        # Run event loop
        self.running = True
        try:
            self.display.run()
        except KeyboardInterrupt:
            logger.info("Interrupted")
        except Exception as e:
            logger.error(f"Error in main loop: {e}")
        
        # Cleanup
        self.cleanup()
        return 0
    
    def stop(self):
        """Stop the compositor"""
        logger.info("Stopping Canvas...")
        self.running = False
        self.display.terminate()
    
    def cleanup(self):
        """Clean up resources"""
        logger.info("Cleaning up...")
        self.display.destroy()


def main():
    """Entry point"""
    # Check if running from TTY
    if "DISPLAY" in os.environ:
        logger.warning("X11 display detected. Canvas should run from TTY for best results.")
        logger.info("You can test with: WLR_BACKENDS=headless or run from TTY1")
    
    # Create and run compositor
    canvas = Canvas()
    return canvas.run()


if __name__ == "__main__":
    sys.exit(main())