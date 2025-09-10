#!/usr/bin/env python3
"""
Debug version of SPOC with visible elements
"""

import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk, Gdk
import sys

class DebugSPOC(Gtk.Window):
    def __init__(self):
        super().__init__(title="SPOC Debug")
        self.fullscreen()
        
        # White background to see elements
        self.override_background_color(Gtk.StateType.NORMAL, Gdk.RGBA(1, 1, 1, 1))
        
        # Main container
        main_box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
        self.add(main_box)
        
        # Top label to confirm rendering
        top_label = Gtk.Label(label="SPOC CONVERSATION AREA")
        top_label.set_size_request(-1, 100)
        main_box.pack_start(top_label, True, True, 0)
        
        # Input area at bottom
        input_frame = Gtk.Frame()
        input_frame.set_label("TYPE HERE:")
        
        self.input = Gtk.Entry()
        self.input.set_size_request(600, 50)
        self.input.set_text("Test input - press Enter to submit")
        self.input.connect("activate", self.on_submit)
        
        input_frame.add(self.input)
        
        # Center the input
        input_box = Gtk.Box(orientation=Gtk.Orientation.HORIZONTAL)
        input_box.pack_start(Gtk.Box(), True, True, 0)  # Left spacer
        input_box.pack_start(input_frame, False, False, 0)
        input_box.pack_start(Gtk.Box(), True, True, 0)  # Right spacer
        
        main_box.pack_start(input_box, False, False, 20)
        
        # Exit instruction
        exit_label = Gtk.Label(label="Press Alt+Escape to exit")
        main_box.pack_start(exit_label, False, False, 10)
        
        # Connect exit
        self.connect("key-press-event", self.on_key_press)
        self.connect("destroy", Gtk.main_quit)
        
    def on_key_press(self, widget, event):
        if event.keyval == Gdk.KEY_Escape and (event.state & Gdk.ModifierType.MOD1_MASK):
            Gtk.main_quit()
            return True
        return False
        
    def on_submit(self, widget):
        text = self.input.get_text()
        print(f"Submitted: {text}")
        self.input.set_text("")

# Run
app = DebugSPOC()
app.show_all()
Gtk.main()