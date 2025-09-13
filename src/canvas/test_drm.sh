#!/bin/bash

echo "Testing DRM device access..."
echo "Current user: $(whoami)"
echo "Groups: $(groups)"
echo ""

echo "DRM devices:"
ls -la /dev/dri/
echo ""

echo "Testing read access to /dev/dri/card0:"
if [ -r /dev/dri/card0 ]; then
    echo "  ✓ Can read"
else
    echo "  ✗ Cannot read"
fi

echo "Testing write access to /dev/dri/card0:"
if [ -w /dev/dri/card0 ]; then
    echo "  ✓ Can write"
else
    echo "  ✗ Cannot write"
fi
echo ""

echo "Checking for seat/session management:"
if command -v loginctl &> /dev/null; then
    echo "  loginctl available:"
    loginctl show-session $(loginctl | grep $(whoami) | awk '{print $1}') 2>/dev/null | grep -E "Type|State|Class"
else
    echo "  loginctl not available"
fi

if [ -e /run/seatd.sock ]; then
    echo "  seatd socket found"
else
    echo "  seatd socket not found"
fi
echo ""

echo "Environment variables:"
echo "  XDG_SESSION_TYPE=$XDG_SESSION_TYPE"
echo "  XDG_RUNTIME_DIR=$XDG_RUNTIME_DIR"
echo "  DISPLAY=$DISPLAY"
echo "  WAYLAND_DISPLAY=$WAYLAND_DISPLAY"