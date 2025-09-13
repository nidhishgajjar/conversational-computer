#!/bin/bash

# Script to run Canvas-DRM with proper permissions

echo "Canvas-DRM Test Runner"
echo "====================="

# Check if we're in a TTY
if [ ! -t 0 ]; then
    echo "Warning: Not running in a TTY. Display output may not work."
fi

# Options to try
echo ""
echo "Testing different approaches:"
echo ""

echo "1. Testing direct run (may fail with permission denied):"
./target/debug/canvas-drm-direct 2>&1 | head -20

echo ""
echo "---"
echo ""

echo "2. To run with DRM master access, you need to:"
echo "   a) Switch to a TTY (Ctrl+Alt+F2-F6)"
echo "   b) Login as aios"
echo "   c) Run: ./target/debug/canvas-drm-direct"
echo ""
echo "3. Alternative: Run with sudo (not recommended for production):"
echo "   sudo ./target/debug/canvas-drm-direct"
echo ""
echo "4. In UTM VM, the graphics may be virtualized. Check:"
lspci 2>/dev/null | grep -i vga || echo "   No PCI VGA device (using virtio-gpu?)"
ls -la /dev/dri/
echo ""
echo "Current display server status:"
ps aux | grep -E "(Xorg|wayland|weston)" | grep -v grep || echo "   No display server running"