#!/bin/bash
#
# Canvas Compositor Launcher
# Runs Canvas in different modes for testing and production
#

CANVAS_PATH="/home/aios/aios/src/canvas/compositor.py"

case "${1:-tty}" in
    "headless")
        echo "Running Canvas in headless mode (testing)..."
        export WLR_BACKENDS=headless
        export WLR_LIBINPUT_NO_DEVICES=1
        python3 "$CANVAS_PATH"
        ;;
    
    "nested")
        echo "Running Canvas nested (under cage)..."
        if ! command -v cage &> /dev/null; then
            echo "Error: cage not installed. Run: sudo pacman -S cage"
            exit 1
        fi
        cage python3 "$CANVAS_PATH"
        ;;
    
    "x11")
        echo "Running Canvas in X11 backend mode..."
        export WLR_BACKENDS=x11
        python3 "$CANVAS_PATH"
        ;;
    
    "tty")
        echo "Running Canvas on TTY (production mode)..."
        # Check if on TTY
        if [ -z "$DISPLAY" ] && [ -z "$WAYLAND_DISPLAY" ]; then
            export WLR_BACKENDS=drm
            export WLR_SESSION=direct
            python3 "$CANVAS_PATH"
        else
            echo "Error: Not on TTY. Switch to TTY1 with Ctrl+Alt+F1"
            echo "Or use: $0 nested"
            exit 1
        fi
        ;;
    
    "debug")
        echo "Running Canvas with debug logging..."
        export WAYLAND_DEBUG=1
        export WLR_LOG_LEVEL=debug
        export WLR_BACKENDS="${WLR_BACKENDS:-headless}"
        python3 "$CANVAS_PATH"
        ;;
    
    *)
        echo "Usage: $0 [headless|nested|x11|tty|debug]"
        echo ""
        echo "Modes:"
        echo "  headless - Test mode without real display"
        echo "  nested   - Run under cage compositor"
        echo "  x11      - Run in X11 window (needs X server)"
        echo "  tty      - Production mode on real TTY"
        echo "  debug    - Enable debug logging"
        exit 1
        ;;
esac