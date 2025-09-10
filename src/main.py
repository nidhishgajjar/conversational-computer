#!/usr/bin/env python3
"""
SPOC - Main entry point for the Conversational Computer
Runs in GTK mode for development, will switch to Canvas compositor later
"""

import gi
gi.require_version('Gtk', '3.0')
gi.require_version('Gdk', '3.0')
from gi.repository import Gtk, Gdk, GLib
import sys
import os

# Set display if not set
if 'DISPLAY' not in os.environ:
    os.environ['DISPLAY'] = ':0'

class SPOCInterface(Gtk.Window):
    def __init__(self):
        super().__init__(title="SPOC")
        
        # Fullscreen setup
        self.fullscreen()
        self.set_default_size(1920, 1080)
        
        # Dark theme
        settings = Gtk.Settings.get_default()
        settings.set_property("gtk-application-prefer-dark-theme", True)
        
        # Main container
        self.main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        self.add(self.main_box)
        
        # Conversation area (scrollable)
        self.scrolled = Gtk.ScrolledWindow()
        self.scrolled.set_policy(Gtk.PolicyType.NEVER, Gtk.PolicyType.AUTOMATIC)
        
        self.conversation_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=10)
        self.conversation_box.set_margin_left(20)
        self.conversation_box.set_margin_right(20)
        self.conversation_box.set_margin_top(20)
        self.conversation_box.set_margin_bottom(20)
        
        self.scrolled.add(self.conversation_box)
        self.main_box.pack_start(self.scrolled, True, True, 0)
        
        # Input area container
        input_container = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        input_container.set_margin_left(20)
        input_container.set_margin_right(20)
        input_container.set_margin_bottom(20)
        input_container.set_margin_top(10)
        
        # Center the input with spacers
        input_container.pack_start(Gtk.Box(), True, True, 0)  # Left spacer
        
        # Input text view with frame
        input_frame = Gtk.Frame()
        input_frame.set_shadow_type(Gtk.ShadowType.IN)
        
        self.input_view = Gtk.TextView()
        self.input_view.set_wrap_mode(Gtk.WrapMode.WORD_CHAR)
        self.input_view.set_left_margin(15)
        self.input_view.set_right_margin(15)
        self.input_view.set_top_margin(10)
        self.input_view.set_bottom_margin(10)
        self.input_view.set_size_request(600, -1)  # Fixed width, dynamic height
        
        # Input buffer
        self.input_buffer = self.input_view.get_buffer()
        self.input_buffer.connect("changed", self.on_input_changed)
        
        input_frame.add(self.input_view)
        input_container.pack_start(input_frame, False, False, 0)
        
        input_container.pack_start(Gtk.Box(), True, True, 0)  # Right spacer
        
        self.main_box.pack_start(input_container, False, False, 0)
        
        # Apply CSS styling
        self.apply_styles()
        
        # Setup keybindings
        self.setup_keybindings()
        
        # Connect signals
        self.connect("key-press-event", self.on_key_press)
        self.connect("destroy", Gtk.main_quit)
        
        # Focus on input
        self.input_view.grab_focus()
        
    def apply_styles(self):
        """Apply CSS for Spotlight-like appearance"""
        css_provider = Gtk.CssProvider()
        css = b"""
        window {
            background: linear-gradient(135deg, #0f1419 0%, #1a1f2e 100%);
        }
        
        textview {
            background-color: rgba(30, 35, 45, 0.95);
            color: #e0e0e0;
            font-family: "SF Mono", "Monaco", monospace;
            font-size: 14px;
            border-radius: 8px;
        }
        
        textview:focus {
            background-color: rgba(35, 40, 50, 0.98);
            box-shadow: 0 0 0 2px rgba(100, 150, 255, 0.3);
        }
        
        frame {
            border: 1px solid rgba(100, 150, 255, 0.2);
            border-radius: 8px;
            background-color: transparent;
        }
        
        .conversation-block {
            background-color: rgba(25, 30, 40, 0.8);
            border-radius: 6px;
            padding: 10px;
            margin: 5px;
        }
        """
        css_provider.load_from_data(css)
        
        screen = Gdk.Screen.get_default()
        style_context = Gtk.StyleContext()
        style_context.add_provider_for_screen(
            screen,
            css_provider,
            Gtk.STYLE_PROVIDER_PRIORITY_APPLICATION
        )
        
    def setup_keybindings(self):
        """Setup Mac-like keybindings"""
        # This is handled in on_key_press for more control
        pass
        
    def on_key_press(self, widget, event):
        """Handle key press events with Mac-like bindings"""
        keyval = event.keyval
        state = event.state
        
        ctrl = (state & Gdk.ModifierType.CONTROL_MASK)
        shift = (state & Gdk.ModifierType.SHIFT_MASK)
        alt = (state & Gdk.ModifierType.MOD1_MASK)
        
        # Escape in fullscreen
        if keyval == Gdk.KEY_Escape and alt:
            Gtk.main_quit()
            return True
            
        # Enter handling
        if keyval == Gdk.KEY_Return:
            if shift:
                # Shift+Enter: new line (default behavior)
                return False
            else:
                # Enter: submit
                self.submit_input()
                return True
                
        # Let TextView handle other keybindings
        return False
        
    def on_input_changed(self, buffer):
        """Auto-resize input area based on content"""
        lines = buffer.get_line_count()
        # Limit to 10 lines visible
        height = min(lines * 25 + 20, 250)
        self.input_view.set_size_request(600, height)
        
    def submit_input(self):
        """Process input submission"""
        start = self.input_buffer.get_start_iter()
        end = self.input_buffer.get_end_iter()
        text = self.input_buffer.get_text(start, end, True).strip()
        
        if not text:
            return
            
        # Add to conversation
        self.add_conversation_block("You", text)
        
        # Clear input
        self.input_buffer.set_text("")
        
        # Simulate response (replace with actual SPOC logic)
        GLib.timeout_add(500, self.add_ai_response, text)
        
    def add_conversation_block(self, sender, message):
        """Add a conversation block"""
        block = Gtk.Box(orientation=Gtk.Orientation.VERTICAL, spacing=5)
        block.get_style_context().add_class("conversation-block")
        
        # Sender label
        sender_label = Gtk.Label(label=sender, xalign=0)
        sender_label.set_markup(f"<b>{sender}</b>")
        block.pack_start(sender_label, False, False, 0)
        
        # Message label
        message_label = Gtk.Label(label=message, xalign=0)
        message_label.set_line_wrap(True)
        message_label.set_selectable(True)
        block.pack_start(message_label, False, False, 0)
        
        self.conversation_box.pack_start(block, False, False, 0)
        block.show_all()
        
        # Auto-scroll to bottom
        GLib.idle_add(self.scroll_to_bottom)
        
    def add_ai_response(self, user_input):
        """Add AI response (placeholder)"""
        response = f"I heard you say: '{user_input}'. SPOC is still learning to respond properly."
        self.add_conversation_block("SPOC", response)
        return False
        
    def scroll_to_bottom(self):
        """Scroll conversation to bottom"""
        adj = self.scrolled.get_vadjustment()
        adj.set_value(adj.get_upper() - adj.get_page_size())
        return False


def main():
    """Main entry point"""
    # Check if GTK can initialize
    if not Gtk.init_check()[0]:
        print("Error: Cannot initialize GTK. Make sure X is running.")
        print("DISPLAY:", os.environ.get('DISPLAY', 'not set'))
        sys.exit(1)
    
    app = SPOCInterface()
    app.show_all()
    Gtk.main()


if __name__ == "__main__":
    main()