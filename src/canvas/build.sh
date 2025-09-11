#!/bin/bash
#
# Build Canvas compositor
#

echo "Building Canvas compositor..."

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust not installed"
    echo "Install with: sudo pacman -S rust cargo"
    echo "Or: curl --proto='=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Build
cd "$(dirname "$0")"
cargo build --release

if [ $? -eq 0 ]; then
    echo "Build successful!"
    echo "Binary at: target/release/canvas"
    echo ""
    echo "Run with:"
    echo "  ./target/release/canvas          # Windowed mode"
    echo "  sudo ./target/release/canvas     # TTY mode (from console)"
else
    echo "Build failed"
    exit 1
fi