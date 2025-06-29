# Phase 0: Basic Window & Rendering

This document tracks the implementation of the foundational window and rendering system.

## Status Overview

- [x] Basic Window Creation
- [x] Metal device and command queue setup
- [ ] Connect Metal device to layer
- [ ] Basic render pipeline
- [ ] Shader compilation
- [ ] First triangle rendering
- [ ] Basic input handling
- [ ] Quad rendering

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

### 3. Connect Metal Device to Layer

- [ ] Set Metal device on the layer
- [ ] Configure drawable properties
- [ ] Set up proper color space

### 4. Metal Renderer Module

- [ ] Create `src/renderer/mod.rs`
- [ ] Create `src/renderer/metal.rs`
- [ ] Basic renderer struct with:
  - Command buffer management
  - Drawable acquisition
  - Present logic

### 5. Shader Setup

- [ ] Create `shaders/` directory
- [ ] Write minimal vertex shader:
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

- [ ] Compile shaders (either at build time or runtime)
- [ ] Create pipeline state object

### 6. First Triangle

- [ ] Define vertex structure:
  ```rust
  #[repr(C)]
  struct Vertex {
      position: [f32; 2],
      color: [f32; 4],
  }
  ```

- [ ] Create vertex buffer with triangle data
- [ ] Implement basic render loop:
  - Get drawable
  - Create command buffer
  - Create render encoder
  - Set pipeline state
  - Set vertex buffer
  - Draw
  - Present

### 7. Basic Input

- [ ] Convert NSEvent to our event types
- [ ] Mouse position tracking
- [ ] Mouse button events
- [ ] Key press events
- [ ] Modifier keys (Shift, Ctrl, Cmd, Alt)

### 8. Quad Rendering

- [ ] Extend vertex buffer for quads (two triangles)
- [ ] Index buffer for efficiency
- [ ] Texture coordinate support
- [ ] Multiple quads in single draw call

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
1. Window opens reliably
2. Triangle renders with vertex colors
3. Mouse position is tracked
4. Basic keyboard input works
5. Multiple colored quads can be rendered in a single frame

## Notes

- Keep everything as simple as possible
- No abstractions yet - direct Metal API usage
- No immediate mode API yet - just foundational rendering
- Focus on getting pixels on screen
