# Pragmatic Architecture - Conversational Computer

## Core Philosophy: Build What Matters, Use What's Solved

This document defines our pragmatic approach to building the Conversational Computer. We focus our innovation where it matters and leverage proven solutions elsewhere.

## The Litmus Test

For every component, ask:
1. **Is this our differentiator?** → Build it
2. **Has this been solved well?** → Use a library  
3. **Will rebuilding this help users?** → Probably not

## What We Build vs What We Use

### BUILD - Our Core Innovation (2000-3000 lines)

These are our differentiators. We own these completely:

| Component | Why Build | What It Does |
|-----------|-----------|--------------|
| **Conversation Engine** | Our core IP | Manages conversation flow, block ordering, context |
| **Block System** | Our innovation | Not windows/apps - conversation artifacts |
| **SPOC Conductor** | Our differentiator | AI orchestration, intent understanding |
| **Layout Engine** | Unique to us | Conversation-first layout, not desktop metaphor |
| **Transition System** | Our UX | Smooth conversation flows, block animations |

### USE - Solved Problems (Libraries)

Don't reinvent these. Use best-in-class solutions:

| Need | Library Options | Why Not Build |
|------|----------------|---------------|
| **GPU Rendering** | Skia, wgpu, Cairo | Decades of optimization |
| **Text Editing** | Xi-rope, Ropey, CodeMirror | 10K+ lines for marginal benefit |
| **Web Rendering** | WebKitGTK, Servo | Impossible scope (10M+ lines) |
| **Video Playback** | GStreamer, FFmpeg | Codec patents, optimization |
| **Font Rendering** | FreeType, HarfBuzz | Complex shaping, i18n |
| **Syntax Highlighting** | Tree-sitter | Already perfect |

## Block Architecture

### Native Blocks

Purpose-built for our system, maximum performance:

```rust
pub struct NativeBlock {
    // Renders directly via Skia/wgpu
    // Full control over every pixel
    // Optimized for conversation
}

Examples:
- Text/Conversation blocks (our current code)
- Code editor (using Xi-rope core)
- System status blocks
- Media player (using GStreamer)
```

### Web Blocks

For compatibility and rapid development:

```rust
pub struct WebBlock {
    webview: WebKitGTK,
    // Can inject JS to control
    // Can hide UI elements
    // AI operates it programmatically
}

Examples:
- Social media (X, Instagram)
- Streaming (Netflix, YouTube)
- Productivity (Google Docs)
- Any web service
```

### Hybrid Blocks

Native UI with web service APIs:

```rust
pub struct HybridBlock {
    native_ui: SkiaCanvas,      // Our rendering
    service_api: SlackAPI,       // Their data
    // Best of both worlds
}

Examples:
- Slack (native UI + Slack API)
- Spotify (native player + Spotify API)
- Email (native UI + IMAP/SMTP)
```

## Block SDK Design

### For Developers

```rust
// Simple trait-based API
pub trait Block {
    // Lifecycle
    fn init(&mut self, config: BlockConfig) -> Result<()>;
    fn destroy(&mut self);
    
    // Rendering
    fn render(&mut self, canvas: &mut Canvas) -> Result<()>;
    
    // Input (when focused)
    fn handle_input(&mut self, event: InputEvent) -> BlockResponse;
    
    // AI interaction
    fn handle_intent(&mut self, intent: Intent) -> Result<()>;
    
    // Capabilities
    fn capabilities(&self) -> Vec<Capability>;
}
```

### Block Manifest

```toml
[block]
name = "spotify-player"
version = "1.0.0"
type = "hybrid"  # native, web, or hybrid

[capabilities]
network = true
audio = true
gpu = false

[intents]
# What this block can handle
patterns = [
    "play music",
    "play {song} by {artist}",
    "pause music",
    "skip song"
]
```

## Example User Flows

### Flow 1: "Show me Elon's tweets from last week"

```
User Input → SPOC understands intent
    ↓
SPOC creates WebBlock(X.com)
    ↓
Injects JS to hide UI chrome
    ↓
Navigates to Elon's profile
    ↓
Scrolls to last week
    ↓
Shows in conversation with transitions
```

### Flow 2: "Show Slack messages from #general"

```
User Input → SPOC understands intent
    ↓
SPOC creates HybridBlock(Slack)
    ↓
Uses Slack API for data
    ↓
Renders native UI (faster, branded)
    ↓
Shows in conversation
    ↓
User can interact directly if needed
```

### Flow 3: "Edit my config file"

```
User Input → SPOC understands intent
    ↓
SPOC creates NativeBlock(Editor)
    ↓
Loads file into Xi-rope
    ↓
Renders with syntax highlighting
    ↓
Full native performance
```

## Transition System

All blocks support smooth transitions:

```rust
pub enum Transition {
    // Conversation flows
    SlideUp(Duration),      // New block appears
    FadeIn(Duration),        // Gentle appearance
    Expand(Duration),        // Block grows to show more
    
    // Focus changes
    Focus(BlockId),          // Highlight active block
    Blur(BlockId),           // Dim inactive blocks
    
    // Layout changes
    Reflow,                  // Conversation reflows
    Fullscreen(BlockId),     // Block takes over
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
- Canvas with Skia/wgpu rendering
- Basic block system
- Native text blocks
- Simple transitions

### Phase 2: Web Blocks (Week 3)
- WebKitGTK integration
- JS injection system
- Web block examples (X, Netflix)

### Phase 3: Hybrid Blocks (Week 4)
- API integration framework
- Native UI templates
- Slack, Spotify examples

### Phase 4: SDK & Store (Week 5-6)
- Developer SDK
- Block packaging
- Block store infrastructure
- Documentation

## Technology Choices (To Evaluate)

### Rendering
- **Option 1: Skia** - Chrome/Flutter's renderer, proven
- **Option 2: wgpu** - Modern, Rust-native, future-proof
- **Option 3: Cairo** - Simple, widespread, stable

### Web Engine
- **Option 1: WebKitGTK** - Linux standard, good enough
- **Option 2: Servo** - Rust native, but incomplete
- **Option 3: CEF** - Chromium, best compatibility

### Text Editing
- **Option 1: Xi-rope** - Rust, modern architecture
- **Option 2: Ropey** - Simple, pure Rust
- **Option 3: CodeMirror** - If we go web-based

## Why This Works

1. **We can ship** - Not stuck building text editors
2. **We innovate where it matters** - Conversation paradigm
3. **We're fast** - GPU acceleration, native performance
4. **We're compatible** - Web blocks for everything
5. **We own our IP** - Core innovation is ours

## The Apple Parallel

Like Apple, we:
- Use BSD (don't write kernel) → We use Skia (don't write renderer)
- Use WebKit (don't write browser) → We use WebKitGTK
- Own the UX completely → We own conversation paradigm

## Success Metrics

- **Ship in 3 months** not 3 years
- **2000 lines of innovation** not 20,000 lines of reinvention
- **60 FPS animations** with GPU acceleration
- **Any web service works** via web blocks
- **Developers can extend** via SDK

## Next Steps

1. Evaluate and choose rendering library (Skia vs wgpu)
2. Prototype WebKitGTK integration
3. Design concrete block examples
4. Build transition system
5. Create developer SDK

---

*This document is our north star. When in doubt, ask: "Is this our differentiator?" If not, use a library and move on.*