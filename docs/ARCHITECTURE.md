# Architecture Decisions Record (ADR)

## Overview
This document captures all architectural decisions made for the Conversational Computer project, explaining the rationale behind each choice.

## System Foundation

### Operating System: Arch Linux
**Decision**: Use Arch Linux as the base OS

**Rationale**:
- **Rolling release**: Always latest packages, no version migrations
- **Minimal by default**: No bloat, only what we need (308 packages)
- **AUR access**: Community packages if needed
- **Documentation**: Excellent wiki and community
- **Customizable**: Build exactly what we want

**Trade-offs**:
- Requires more setup vs Ubuntu/Debian
- Less "stable" than fixed releases
- Smaller enterprise adoption

### No Desktop Environment
**Decision**: Boot directly to Canvas compositor, no traditional desktop

**Rationale**:
- **Paradigm shift**: Conversation is the interface, not windows
- **Resource efficiency**: More RAM/CPU for AI processing
- **Simpler**: No window management complexity
- **Focused**: Users interact through conversation only

**Trade-offs**:
- No fallback to traditional GUI
- Must implement everything ourselves
- Learning curve for users

## Architecture Layers

### 1. Hardware Layer
- Standard x86_64 hardware
- Framebuffer device (`/dev/fb0`)
- Input devices via TTY

### 2. Kernel Layer
- Linux kernel 6.16.5
- Direct framebuffer access
- TTY for input handling

### 3. Canvas Compositor Layer
**Decision**: Custom Wayland compositor in Rust

**Rationale**:
- **Performance**: Rust has zero-cost abstractions
- **Safety**: Memory safety without garbage collection
- **Modern**: Built for systems programming
- **Direct control**: No unnecessary abstraction layers

**Implementation Evolution**:
1. Started with Python + wlroots (failed - binding issues)
2. Tried GTK prototype (worked but not pure vision)
3. Settled on Rust + direct framebuffer (working)
4. Future: Full Wayland compositor with Smithay

### 4. SPOC Conductor Layer
**Decision**: SPOC as the orchestration layer

**Rationale**:
- **Single responsibility**: SPOC conducts, Canvas renders
- **Modularity**: Can swap AI backends
- **Clarity**: Clear separation of concerns

### 5. Conversation Layer
**Decision**: Blocks as conversation units (not windows/apps)

**Rationale**:
- **Natural**: Matches how humans converse
- **Contextual**: Each block maintains context
- **Flexible**: Can represent any interaction type

## Technical Decisions

### Framebuffer Rendering
**Decision**: Direct framebuffer manipulation for initial implementation

**Rationale**:
- **Simplicity**: No GPU complexity initially
- **Learning**: Understand rendering fundamentals
- **Control**: Pixel-perfect control
- **Portable**: Works on any Linux with framebuffer

**Code Structure**:
```rust
/dev/fb0 -> Memory map -> Pixel buffer -> Screen
```

### Text Rendering: Bitmap Fonts
**Decision**: 8x8 bitmap font for initial text rendering

**Rationale**:
- **Simple**: No font libraries needed
- **Fast**: Direct pixel manipulation
- **Sufficient**: Good enough for prototype
- **Educational**: Understand text rendering basics

**Future**: FreeType or HarfBuzz for production

### Input Handling: Raw TTY Mode
**Decision**: Terminal in raw mode for keyboard input

**Rationale**:
- **Direct**: No input method complexity
- **Control**: Handle every keypress
- **Simple**: Works without X11/Wayland
- **Reliable**: Terminal input is well-understood

**Implementation**:
```rust
TTY -> Raw mode -> Input thread -> Event channel -> Main loop
```

### Language Progression
**Decision**: Python → Rust progression

**Rationale**:
- **Python**: Fast prototyping, failed due to wlroots bindings
- **Rust**: Production performance, memory safety, better ecosystem

### Build System
**Decision**: Cargo for Rust

**Rationale**:
- **Integrated**: Built into Rust ecosystem
- **Reliable**: Dependency management just works
- **Fast**: Incremental compilation
- **Simple**: One command builds everything

## Visual Design Decisions

### Color Scheme
- **Background**: Dark blue-gray (RGB: 20, 25, 35)
- **Input bar**: Darker (RGB: 30, 35, 45)
- **Text**: White (RGB: 255, 255, 255)
- **SPOC prompt**: Light blue (RGB: 100, 200, 255)
- **Borders**: Medium blue (RGB: 100, 150, 255)

**Rationale**: Easy on eyes, professional, good contrast

### Layout
```
┌─────────────────────────────────────┐
│ CANVAS - Conversational Computer    │ <- Title
│                                     │
│ [Conversation flows here]           │ <- Blocks area
│                                     │
│                                     │
│ ┌─────────────────────────────┐    │
│ │ SPOC> [User input here]     │    │ <- Input bar
│ └─────────────────────────────┘    │
└─────────────────────────────────────┘
```

**Rationale**: 
- Input always visible at bottom
- Conversation flows naturally upward
- Centered for focus
- Fixed width (600px) for readability

### Cursor Design
**Decision**: Thin vertical line with blink

**Rationale**:
- **Modern**: Matches current UI expectations
- **Visible**: Clear where input will appear
- **Familiar**: Standard text input indicator

## Development Decisions

### Version Control: Git + GitHub
**Decision**: Git with GitHub hosting

**Rationale**:
- **Standard**: Industry standard VCS
- **Distributed**: Full history locally
- **GitHub**: Free hosting, easy collaboration
- **CI/CD**: GitHub Actions available if needed

### Project Structure
```
aios/
├── src/
│   ├── canvas/     # Compositor
│   ├── spoc/       # Conductor
│   └── blocks/     # Block system
├── docs/           # Documentation
└── tests/          # Test suite
```

**Rationale**: Clear separation, easy to navigate

### Testing Approach
**Decision**: Start without tests, add when stable

**Rationale**:
- **Speed**: Rapid prototyping phase
- **Exploration**: Still discovering requirements
- **Future**: Will add tests once core is stable

## Future Considerations

### GPU Acceleration
- Move from framebuffer to OpenGL/Vulkan
- Hardware acceleration for text rendering
- Smooth animations

### Wayland Compositor
- Full Wayland protocol support
- Other applications can run if needed
- Better hardware integration

### AI Integration
- LLM backend selection
- Local vs cloud processing
- Context management
- Privacy considerations

### Input Methods
- Voice input
- Gesture recognition
- Multi-language support
- Accessibility features

## Rejected Approaches

### X11
**Rejected because**: Legacy, complex, not suited for our paradigm

### Existing Compositors (Sway, KDE, GNOME)
**Rejected because**: Built for window paradigm, too much baggage

### Electron/Web Technologies
**Rejected because**: Too heavy, not native enough for OS-level

### GTK/Qt for Canvas
**Rejected because**: Not direct enough, adds unnecessary layers

## Performance Targets

- **Boot time**: < 5 seconds to SPOC input
- **Input latency**: < 16ms (one frame)
- **Memory usage**: < 100MB for Canvas
- **CPU usage**: < 5% idle

## Security Considerations

- No network access in Canvas
- Input sanitization in SPOC
- Sandboxed AI execution
- Minimal attack surface

## Conclusion

Every decision prioritizes:
1. **Simplicity** over features
2. **Performance** over compatibility  
3. **Control** over convenience
4. **Focus** on conversational paradigm

The architecture is designed to evolve from simple (framebuffer) to complex (full compositor) while maintaining the core vision of conversation as the primary interface.