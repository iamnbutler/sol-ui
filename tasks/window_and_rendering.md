# Phase 0: Basic Window & Rendering

This document tracks the implementation of the foundational window and rendering system.

## Status Overview

- [x] Basic Window Creation
- [x] Metal device and command queue setup
- [x] Connect Metal device to layer
- [x] Basic render pipeline
- [x] Shader compilation
- [x] First triangle rendering
- [x] Basic immediate mode context
- [x] Quad rendering for UI
- [x] UI elements (group/container)
- [x] Color system (palette crate)
- [x] Text rendering (basic implementation with Core Text)
- [ ] Layer system (partially implemented, needs render pass management)
- [ ] Input handling (deferred)

## Completed Tasks

### 1. Basic Window Creation ✅

Created minimal macOS window with Metal layer support:

- **Files created**:

  - `src/platform/mac/window.rs` - Window management with NSWindow/NSView
  - `src/platform/mac/mod.rs` - Platform module organization
  - `src/platform/mod.rs` - Cross-platform interface
  - `src/app.rs` - Application lifecycle management

- **Key decisions**:
  - Minimal `unsafe` blocks - only around actual FFI calls
  - Direct Objective-C runtime usage (no high-level wrappers)
  - CAMetalLayer attached to NSView
  - Window delegate for proper event handling

### 2. Metal Initialization ✅

- Created Metal device and command queue in `App::new()`
- Metal layer created and attached to view
- Connected device to layer with proper configuration
- Retina display support with 2x scale factor

### 3. Connect Metal Device to Layer ✅

- Modified `Window::new` to accept Metal device
- Set device on layer with `layer.set_device(device)`
- Configured drawable properties (opaque, framebuffer_only, presents_with_transaction)
- Set drawable size accounting for Retina displays

### 4. App Polish ✅

- Added `activateIgnoringOtherApps` to bring window to front
- Created minimal macOS menu with just Quit (Cmd+Q)
- Removed unnecessary menu items (YAGNI principle)
- Created `src/platform/mac/menu.rs` for menu handling

### 5. Metal Renderer Module ✅

- Created `MetalRenderer` struct to encapsulate rendering logic
- Moved rendering code from `App` into dedicated renderer
- Handles shader compilation, pipeline state creation, and vertex buffer management
- Updated to support dynamic vertex buffer creation from draw commands

### 6. Shader Setup ✅

- Embedded shaders directly in Rust code
- Runtime shader compilation using Metal Shading Language
- Vertex shader transforms 2D positions to clip space
- Fragment shader passes through vertex colors
- Pipeline state configured for BGRA8Unorm render target

### 7. First Triangle → UI Rendering ✅

- Started with triangle rendering to verify Metal pipeline
- Evolved to full UI quad rendering system
- Converts UI draw commands to vertex data
- Batches all UI elements into single draw call
- Proper coordinate transformation from screen space to NDC

### 8. Immediate Mode UI Foundation ✅

**Created comprehensive immediate mode system:**

- **Files created**:

  - `src/ui/mod.rs` - UI module organization
  - `src/ui/context.rs` - Main UI context for immediate mode
  - `src/ui/draw.rs` - Draw command system and primitives
  - `src/ui/id.rs` - Widget ID system for state tracking

- **Features implemented**:
  - `UiContext` with frame-based state management
  - Hierarchical widget ID system with `IdStack`
  - Draw command batching with `DrawList`
  - Layout helpers (vertical, horizontal, manual positioning)
  - Proper draw ordering (backgrounds before content)

### 9. UI Elements ✅

**Implemented core UI elements:**

- `ui.text()` - Text rendering API (rendering not implemented yet)
- `ui.rect()` - Colored rectangle rendering
- `ui.group()` - Container with optional background
- `ui.vertical()` / `ui.horizontal()` - Layout groups
- `ui.window()` - Window-style container with title bar
- `ui.space()` - Spacing in layout direction

**Draw ordering fixed with insert-at-position system**

### 10. Color System ✅

**Integrated palette crate for robust color handling:**

- Using `Srgba` for proper sRGB color space
- Access to CSS named colors
- Color manipulation capabilities (mix, lighten, darken)
- Type-safe color handling
- Ready for HDR/wide gamut support

## Current Architecture

```rust
// Immediate mode pattern in use
ui.group_styled(Some(named::DARKSLATEGRAY.into()), |ui| {
    ui.text("Hello from Toy UI!");
    ui.horizontal(|ui| {
        ui.rect(vec2(50.0, 50.0), named::CRIMSON.into());
        ui.rect(vec2(50.0, 50.0), named::LIMEGREEN.into());
    });
});
```

## Completed: Basic Text Rendering ✅

- ✅ Created font manager using font-kit
- ✅ Implemented glyph rasterization with proper sizing
- ✅ Created texture atlas for glyph caching
- ✅ Added texture support to Metal renderer
- ✅ Core Text shaping for proper text layout
- ✅ Text measurement for UI layout
- ✅ Full pipeline: shape → rasterize → atlas → render

## Current: Layer System Implementation (In Progress)

The layer system foundation is implemented but needs completion:

### What's Done

- ✅ Created `LayerManager` for managing z-ordered layers
- ✅ Implemented `Layer` trait with render and input handling methods
- ✅ Created `RawLayer` for direct shader access (placeholder)
- ✅ Created `UiLayer` for immediate mode UI
- ✅ Basic layer options (input handling, blend modes)
- ✅ Layer insertion with z-ordering

### What's Needed

- [ ] Fix render pass management - current architecture assumes single pass
- [ ] Implement `render_draw_list` to actually render commands
- [ ] Support multiple render passes per frame
- [ ] Implement blend modes between layers
- [ ] Add clearing behavior for layers
- [ ] Test multi-layer rendering scenarios

### Current Status

The renderer is currently bypassing the layer system due to architectural issues with render pass management. The `render_frame` method needs restructuring to support multiple layers with their own render passes.

## Later Phases

### Input Handling (Deferred)

We're intentionally deferring input to focus on rendering:

- Convert NSEvent to our event types
- Mouse position tracking and hit testing
- Keyboard input and text editing
- Focus management
- Widget interaction states (hover, active, focused)

### Advanced Features

- Clipping for windows and scroll areas
- Animations and transitions
- Taffy integration for advanced layout
- Entity system for persistent widget state
- Layer system for complex compositions
- 3D UI elements in world space

## Success Criteria

Phase 0 is complete when:

1. Window opens reliably ✅
2. Basic Metal rendering works ✅
3. Immediate mode UI context works ✅
4. Can render rectangles and containers ✅
5. Can render text with `ui.text("Hello")` ✅
6. Multiple UI elements render in single frame ✅
7. Proper color handling with palette crate ✅

## Notes

- Successfully avoided premature abstraction
- Clean separation between platform, rendering, and UI layers
- Immediate mode pattern working well
- Text rendering fully implemented with Core Text integration
- Layer system foundation complete but blocked on render architecture
- Next priority: Complete layer system before moving to input handling
