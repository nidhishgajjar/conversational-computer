# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AIOS - A Conversational Computer where conversation is the primary interface, not GUI or CLI. The system consists of three main components:
- **Canvas**: The DRM/KMS-based compositor (Rust) that renders the conversational interface
- **SPOC (Conductor)**: The AI conductor (Python) that orchestrates all interactions and manages conversation flow
- **Blocks**: Units of conversation, not traditional windows or apps

## Current Development Status

The project is in early development, focusing on getting basic DRM rendering working. Check `docs/CURRENT_STATUS.md` for the latest progress and blockers. Currently working on transitioning from Smithay compositor abstractions to direct DRM/GBM rendering.

## Development Commands

### Canvas (Rust Display Component)
```bash
# Build the Canvas component
cd src/canvas
cargo build

# Run Canvas (requires proper privileges for DRM access)
sudo cargo run

# Build release version with optimizations
cargo build --release

# Check for compilation errors without building
cargo check

# Run with debug logging
RUST_LOG=debug cargo run
```

### Conductor/SPOC (Python AI Component)
```bash
# Activate Python virtual environment
source src/conductor/conductor_env/bin/activate

# Run SPOC server
cd src/conductor
python server.py

# Run conductor directly for testing
python conductor.py
```

## Architecture and Key Components

### Canvas (src/canvas/)
- **Purpose**: Direct hardware rendering via DRM/KMS
- **Current State**: Minimal DRM initialization working, attempting to implement basic frame rendering
- **Key Files**:
  - `src/main.rs`: DRM device initialization and rendering loop
  - `src/spoc_client.rs`: Unix socket client for SPOC communication (stub)
- **Dependencies**: Smithay (Wayland compositor toolkit), GBM, EGL/GLES

### Conductor/SPOC (src/conductor/)
- **Purpose**: AI orchestration and conversation management
- **Components**:
  - `server.py`: Unix socket server for Canvas IPC
  - `conductor.py`: Core conversation engine with intent understanding
- **Communication**: Unix socket at `/tmp/spoc.sock`

### Blocks System
- **Design Philosophy**: Conversation artifacts, not windows
- **Types**: Native blocks (max performance), Web blocks (compatibility), Hybrid blocks (native UI + service APIs)
- **Future SDK**: Trait-based API for block development

## Important Development Guidelines

1. **Progress Tracking**: Update `docs/CURRENT_STATUS.md` at major checkpoints (success, failure, or plan changes), not after every edit
2. **Plan Changes**: Always ask for approval before pivoting or changing the development plan
3. **Code Philosophy**: "Keep only what works 100%. Build on solid foundations."
4. **Focus Areas**: Build differentiators (conversation engine, block system, SPOC), use libraries for solved problems (rendering, text editing, web)

## Current Technical Challenges

1. **Smithay API Usage**: Working through proper PlaneState construction for page_flip operations
2. **Virtual GPU Compatibility**: Running in UTM VM with virtio-gpu and llvmpipe software renderer
3. **Frame Presentation**: Need to achieve clean 60 FPS rendering before adding features

## Testing Environment

- Running in UTM virtual machine on macOS
- Virtual display: 1280x800 @ 75Hz
- GPU: virtio-gpu with llvmpipe software renderer
- DRM device: `/dev/dri/card0`

## Next Steps (From CURRENT_STATUS.md)

1. Complete minimal working base with solid color clearing
2. Verify 60 FPS frame presentation
3. Add simple rectangle rendering
4. Implement basic keyboard input
5. Add text rendering

## Documentation Structure

- `README.md`: Project overview and core concepts
- `docs/CURRENT_STATUS.md`: Live development status and blockers
- `docs/PRAGMATIC_ARCHITECTURE.md`: Technical architecture and technology choices
- `docs/BLOCKS_AND_SDK.md`: Block system design and SDK specification
- `docs/conversational-computer-spec.md`: Product specification
- `docs/system-baseline.md`: System requirements

## Development Tips

- When stuck on Smithay APIs, consider direct DRM/KMS approach for more control
- Test rendering changes with simple solid colors first before complex graphics
- Use `RUST_LOG=debug` for detailed debugging output from Canvas
- Monitor `/tmp/spoc.sock` for IPC communication between Canvas and SPOC