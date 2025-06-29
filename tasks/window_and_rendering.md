# Phase 0: Basic Window & Rendering

This document tracks the implementation of the foundational window and rendering system.

## Status Overview

- [x] Basic Window Creation
- [x] Metal device and command queue setup
- [x] Connect Metal device to layer
- [x] Basic render pipeline
- [x] Shader compilation
- [x] First triangle rendering
- [x] Quad rendering (deferred - moving to immediate mode)
- [ ] Basic immediate mode context
- [ ] Text rendering
- [ ] UI elements (group/container)

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

### 2. Metal Initialization ✅ (Partial)

- Created Metal device and command queue in `App::new()`
- Metal layer created and attached to view
- Still need to connect device to layer

## Next Tasks

### 3. Connect Metal Device to Layer ✅

- [x] Set Metal device on the layer
- [x] Configure drawable properties
- [x] Set up proper color space

**Completed**:

- Modified `Window::new` to accept Metal device
- Set device on layer with `layer.set_device(device)`
- Configured drawable properties (opaque, framebuffer_only, presents_with_transaction)
- Set drawable size accounting for Retina displays
- Basic render loop now clears to dark blue color

### 4. App Polish ✅

- [x] Bring window to front on app launch
- [x] Add macOS menu bar

**Completed**:

- Added `activateIgnoringOtherApps` to bring window to front
- Created proper macOS menu structure:
  - App menu with Quit (Cmd+Q)
  - Edit menu with standard operations (Undo, Redo, Cut, Copy, Paste, Select All)
  - Window menu with standard operations (Minimize, Zoom)
- Created `src/platform/mac/menu.rs` for menu handling

### 5. Metal Renderer Module ✅

- [x] Create `src/renderer/mod.rs`
- [x] Create `src/renderer/metal.rs`
- [x] Basic renderer struct with:
  - Command buffer management
  - Drawable acquisition
  - Present logic

**Completed**:

- Created `MetalRenderer` struct to encapsulate rendering logic
- Moved rendering code from `App` into dedicated renderer
- Handles shader compilation, pipeline state creation, and vertex buffer management
- Clean separation of concerns between app lifecycle and rendering

### 6. Shader Setup ✅

- [x] Write minimal vertex shader
- [x] Write minimal fragment shader
- [x] Compile shaders at runtime
- [x] Create pipeline state object

**Completed**:

- Embedded shaders directly in Rust code (no separate files needed yet)
- Runtime shader compilation using Metal Shading Language
- Vertex shader transforms 2D positions to clip space
- Fragment shader passes through vertex colors
- Pipeline state configured for BGRA8Unorm render target

  ```metal
  struct Vertex {
      float2 position [[attribute(0)]];
      float4 color [[attribute(1)]];
  };

  struct VertexOut {
      float4 position [[position]];
      float4 color;
  };

  vertex VertexOut vertex_main(Vertex in [[stage_in]]) {
      VertexOut out;
      out.position = float4(in.position, 0.0, 1.0);
      out.color = in.color;
      return out;
  }
  ```

- [ ] Write minimal fragment shader:

  ```metal
  fragment float4 fragment_main(VertexOut in [[stage_in]]) {
      return in.color;
  }
  ```

### 7. First Triangle ✅

- [x] Define vertex structure
- [x] Create vertex buffer with triangle data
- [x] Implement basic render loop

**Completed**:

```rust
#[repr(C)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}
```

- Defined `Vertex` struct with position (2D) and color (RGBA)
- Created vertex buffer with three colored vertices (red, green, blue)
- Implemented render loop in `MetalRenderer::render_frame()`
- Triangle renders with interpolated colors on dark blue background
- Proper vertex attribute configuration with stride and offsets

## Next: Immediate Mode UI Foundation

### 8. Basic Immediate Mode Context

- [ ] Create UI context struct that tracks frame state
- [ ] Implement basic immediate mode pattern
- [ ] ID generation for widgets
- [ ] Basic layout stack (manual positioning for now)

### 9. Quad/Rectangle Rendering

- [ ] Create quad rendering for UI elements
- [ ] Support solid color fills
- [ ] Batch multiple quads in single draw call
- [ ] Basic clipping support

### 10. Text Rendering

- [ ] Integrate Core Text for font rasterization
- [ ] Create basic glyph atlas
- [ ] Implement `ui.text("Hello")` API
- [ ] Support basic text styling (size, color)

### 11. Group/Container Element

- [ ] Implement `ui.group()` for layout containers
- [ ] Basic styling (background color, padding)
- [ ] Nested groups support
- [ ] Manual positioning within groups

### Later: Input Handling (Deferred)

Input handling will come after we have basic UI elements working:

- Convert NSEvent to our event types
- Mouse position tracking
- Mouse button events
- Key press events
- Modifier keys

## Architecture Notes

### Render Loop Structure

```rust
impl App {
    fn render_frame(&self) {
        // 1. Get next drawable
        let drawable = self.window.metal_layer().next_drawable();

        // 2. Create command buffer
        let command_buffer = self.command_queue.new_command_buffer();

        // 3. Create render pass
        let render_pass = RenderPassDescriptor::new();
        render_pass.color_attachment(0)
            .set_texture(drawable.texture());

        // 4. Encode rendering commands
        let encoder = command_buffer.new_render_command_encoder(&render_pass);
        // ... draw calls ...
        encoder.end_encoding();

        // 5. Present
        command_buffer.present_drawable(&drawable);
        command_buffer.commit();
    }
}
```

### Coordinate System

For Phase 0, we'll use normalized device coordinates:

- X: -1.0 (left) to 1.0 (right)
- Y: -1.0 (bottom) to 1.0 (top)
- Z: 0.0 (near) to 1.0 (far)

Later phases will add proper projection matrices and our unit types.

## Success Criteria

Phase 0 is complete when:

1. Window opens reliably ✅
2. Triangle renders with vertex colors ✅
3. Basic immediate mode UI context works
4. Can render text with `ui.text("Hello")`
5. Can create styled containers with `ui.group()`
6. Multiple UI elements can be rendered in a single frame

## Notes

- Keep everything as simple as possible
- No abstractions yet - direct Metal API usage
- No immediate mode API yet - just foundational rendering
- Focus on getting pixels on screen
