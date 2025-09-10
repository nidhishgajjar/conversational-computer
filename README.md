# AIOS - Conversational Computer

An AI-first operating system where conversation is the primary interface.

## Core Concepts

- **CUI** (Conversational UI) - Not GUI, not CLI, but conversation-first
- **SPOC** - The conductor that orchestrates all interactions  
- **Canvas** - The compositor that renders our conversational interface
- **Blocks** - Units of conversation (not windows or apps)

## Architecture

```
Canvas (Compositor) 
    └── SPOC (Conductor)
         └── Input Bar (fixed bottom)
         └── Conversation Area (blocks flow above)
```

## Development Plan

1. **Phase 1** (Current): Python prototype with GTK
2. **Phase 2**: Custom Wayland compositor in Python
3. **Phase 3**: Production rewrite in Rust

## Project Structure

```
aios/
├── src/
│   ├── spoc/          # SPOC conductor
│   ├── canvas/        # Compositor  
│   └── blocks/        # Block system
├── docs/              # Documentation
└── tests/             # Test suite
```

## Running

```bash
# Development mode (GTK)
python3 src/main.py

# Compositor mode (future)
python3 src/canvas/compositor.py
```