#!/usr/bin/env python3
"""
Super simple GUI test - just a window with text
"""

import gi
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk
import sys

# Create window
win = Gtk.Window(title="SPOC Test")
win.set_default_size(800, 600)
win.connect("destroy", Gtk.main_quit)

# Add label
label = Gtk.Label(label="SPOC IS RUNNING!\n\nPress Alt+F4 or close button to exit")
win.add(label)

# Show
win.show_all()
Gtk.main()