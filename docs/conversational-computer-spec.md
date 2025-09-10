# Conversational Computer Specification

## Core Terminology

### System Level
- **Conversational Computer** - An AI-first operating system paradigm where conversation is the primary interface
- **CUI** (Conversational User Interface) - The interaction paradigm (vs GUI/CLI)

### Components
- **Canvas** - The Wayland compositor that provides the visual foundation
- **SPOC** - The conductor that orchestrates all interactions and conversations
- **Blocks** - Units of conversation/interaction (replaces windows/apps concept)

## Architecture

```
Hardware
    └── Linux Kernel (Arch)
        └── Canvas (Wayland Compositor)
            └── SPOC (Conductor)
                ├── Input Bar (fixed bottom)
                ├── Conversation Area (blocks flow)
                └── Block Manager
```

## Design Principles

1. **Conversation First** - No desktop, no windows, just conversation
2. **Always Ready** - Input bar always visible, always active
3. **Context Aware** - Blocks maintain conversation context
4. **AI Native** - Built for AI interaction from ground up

## Visual Design

- **Background**: Solid color or gradient (no desktop metaphor)
- **Input Bar**: 
  - Fixed at bottom center
  - Spotlight-like appearance
  - Multi-line capable but starts as single line
  - 600px width, auto-height
  
- **Conversation Area**:
  - Flows from bottom to top
  - Blocks appear above input
  - Scrollable when needed
  
## Keybindings (Mac-like)

- `Enter` - Submit message
- `Shift+Enter` - New line
- `Cmd+Left/Right` - Jump to line start/end  
- `Option+Left/Right` - Jump words
- `Cmd+Shift+Up/Down` - Select all above/below
- `Option+Shift+Left/Right` - Select words
- `Cmd+Shift+Left/Right` - Select lines
- `Option+Delete` - Delete word
- `Cmd+Delete` - Delete line
- `Option+Escape` - System menu/exit

## Implementation Plan

### Phase 1: Canvas Compositor (Python)
- Minimal Wayland compositor (~1000 lines)
- Direct rendering to framebuffer
- No window management needed
- Handle input/output directly

### Phase 2: SPOC Integration
- Text rendering system
- Input handling
- Block rendering
- Conversation flow

### Phase 3: AI Integration
- LLM backend connection
- Context management
- Block intelligence

### Phase 4: Production (Rust)
- Rewrite for performance
- ~2000 lines estimated
- Same architecture

## Canvas Compositor Requirements

### Core Features
1. Wayland protocol support
2. DRM/KMS for display
3. Input handling (keyboard/mouse)
4. OpenGL ES rendering
5. Text rendering (FreeType/Pango)

### Minimal Dependencies
- wlroots (or direct Wayland)
- libinput
- OpenGL ES
- FreeType/Cairo for text
- No GTK/Qt needed

### Startup Flow
1. Initialize Wayland display
2. Setup DRM backend
3. Create EGL context
4. Initialize input
5. Render background
6. Create SPOC input area
7. Event loop

## Development Environment

- **OS**: Arch Linux (minimal)
- **Display**: Direct framebuffer/DRM
- **Language**: Python → Rust
- **No Desktop Environment**: Boot directly to Canvas

## File Structure

```
/
├── /usr/local/bin/
│   └── canvas           # Compositor executable
├── /etc/
│   ├── canvas/
│   │   └── config.toml  # Configuration
│   └── systemd/system/
│       └── canvas.service
└── /opt/spoc/
    ├── conductor/       # SPOC logic
    ├── blocks/          # Block system
    └── themes/          # Visual themes
```

## Next Steps

1. Clean system to vanilla state
2. Implement Canvas compositor with basic rendering
3. Add SPOC input handling
4. Implement block system
5. Connect AI backend