# Toy Immediate Mode UI Framework Plan

## Overview

Building a Mac-only immediate mode UI framework with Metal rendering, supporting 3D by default.

### Key Principles

- **Immediate Mode**: No retained widget state, UI is redrawn every frame
- **Metal-based**: Direct Metal API usage for rendering
- **3D First**: Built with 3D support from the ground up using `glam`
- **No Shortcuts**: Direct macOS API calls with `repr(C)` structs, no helper crates
- **Reference**: Study gpui's architecture but build our own approach

## Architecture

### Core Components

1. **Window Management** ✅

   - Direct Cocoa/AppKit integration
   - NSWindow and NSView creation
   - Event handling (mouse, keyboard, touch) - _input deferred_
   - Basic frame loop

2. **Metal Renderer** ✅ (partial)

   - Command queue and buffer management ✅
   - Shader pipeline (vertex + fragment shaders) ✅
   - Texture atlas for UI elements - _needed for text_
   - 3D transformation matrices - _future_
   - Depth testing and blending - _future_
   - Multi-pass rendering for layers - _future_

3. **Layer System**

   - Ordered rendering layers (by z-index)
   - Two layer types:
     - **Raw layers**: Direct vertex/fragment shader access
     - **UI layers**: Taffy-managed layout with immediate mode API
   - Depth buffer management between layers
   - Compositing and blending
   - **Input handling**:
     - Z-order determines input priority (topmost opted-in layer gets input first)
     - Layers must explicitly opt-in to receive input
     - UI layers can consume input when opted-in
     - Raw layers initially won't handle input (can be added later)
     - Modal dialogs naturally work by having higher z-order

4. **Immediate Mode Core** ✅ (partial)

   - Frame-based UI state ✅
   - ID system for widget continuity ✅
   - Input handling and focus management - _deferred_
   - Taffy tree construction for UI layers - _future_
   - Layout caching and invalidation - _future_

5. **Entity System**

   - Type-safe persistent state across frames
   - Reference counted with automatic cleanup
   - Allows stateful widgets in immediate mode context
   - Weak references for non-owning handles
   - Change observation/notification

6. **Math & Geometry**

   - Use `glam` for vectors, matrices, quaternions
   - Bounds, rects, and 3D volumes
   - Transformation hierarchies
   - Camera and projection systems

7. **Drawing Primitives**
   - Quads (textured and colored)
   - Text rendering (Core Text integration)
   - Paths and shapes
   - 3D meshes
   - Shadows and effects

## Implementation Phases

### Phase 1: Foundation ✅

- [x] Basic Rust project setup with glam and taffy dependencies
- [x] macOS window creation with Metal layer
- [x] Basic Metal renderer initialization
- [x] Simple quad rendering
- [x] Event loop (input handling deferred)

### Phase 2: Layer System ✅

- [x] Layer management and ordering
- [x] Render pass per layer
- [x] Raw layer implementation (direct shader access)
- [x] UI layer scaffolding
- [x] Input event routing between layers

### Phase 3: Immediate Mode Core ✅

- [x] Widget ID system
- [x] Basic UI context struct
- [x] Frame state management
- [x] Hot/active/focused widget tracking
- [x] Taffy tree construction for UI layers
- [x] Layout result caching
- [x] Entity system for persistent state
- [x] Entity lifetime management

### Phase 4: Basic Widgets (In Progress)

- [x] Container (group with background)
- [x] Layout helpers (vertical/horizontal)
- [x] Label (text rendering implemented)
- [x] Button (with hover/press states)
- [x] Checkbox (with focus support)
- [ ] Input field
- [ ] Slider

### Phase 5: 3D Features

- [ ] 3D transformation system for raw layers
- [ ] Camera controls
- [ ] 3D mesh rendering in raw layers
- [ ] Mixing 2D UI layers with 3D content
- [ ] Basic lighting for 3D content

### Phase 6: Advanced Features (In Progress)

- [x] Texture atlas management (for glyphs)
- [x] Font atlas and text shaping (Core Text integration)
- [x] Scrollable areas (scroll container with momentum)
- [x] Keyboard shortcuts system
- [x] Undo/redo system (Command pattern)
- [ ] Animations and transitions
- [ ] Nested UI layers
- [ ] Advanced taffy layouts (grid)

### Phase 7: Polish (In Progress)

- [x] Performance optimizations (buffer pooling, text caching)
- [ ] Memory management improvements
- [ ] Debug overlay showing layers
- [ ] Documentation
- [ ] Examples mixing UI and 3D

## Technical Details

### Window System

```rust
#[repr(C)]
struct NSWindow {
    // Objective-C runtime fields
}

#[repr(C)]
struct NSView {
    // Objective-C runtime fields
}
```

### Metal Integration

- Use `metal` crate for Metal API bindings
- Vertex buffer for UI geometry
- Uniform buffer for transforms
- Texture binding for fonts/images

### Coordinate System

Every position in the system must have explicit units attached. This prevents coordinate space mixing bugs at compile time.

#### Unit Types

```rust
#[repr(transparent)]
pub struct LogicalPixel(pub f32);     // Screen space, DPI-independent

#[repr(transparent)]
pub struct PhysicalPixel(pub i32);    // Actual hardware pixels

#[repr(transparent)]
pub struct WorldUnit(pub f32);        // 3D world space units

#[repr(transparent)]
pub struct LocalUnit<Parent = ()>(pub f32, PhantomData<Parent>); // Relative to parent transform
```

#### Geometric Types

```rust
pub struct Position<Unit> {
    pub x: Unit,
    pub y: Unit,
    pub z: Unit,
}

pub struct Size<Unit> {
    pub width: Unit,
    pub height: Unit,
    pub depth: Unit,
}
```

#### Usage

```rust
// Functions declare what coordinate space they expect
impl UI {
    fn button_at(&mut self, pos: Position<LogicalPixel>, text: &str) -> bool;
    fn render_to_framebuffer(&mut self, size: Size<PhysicalPixel>);
}

impl Scene {
    fn place_mesh(&mut self, pos: Position<WorldUnit>, mesh: MeshId);
}

// Conversions are explicit
impl Renderer {
    fn logical_to_physical(&self, logical: LogicalPixel) -> PhysicalPixel {
        PhysicalPixel((logical.0 * self.scale_factor) as i32)
    }
}
```

#### Benefits

- **Compile-time safety**: Can't accidentally pass world coordinates to a UI function
- **Self-documenting**: Function signatures clearly show what space they work in
- **DPI handling**: Distinction between logical and physical pixels is explicit
- **3D integration**: UI layers can exist in world space with proper unit tracking

### Layer System Example

```rust
// Define layers
app.raw_layer(0, |ctx| {
    // Background - raw layer with shader (no input handling)
    ctx.draw_fullscreen_quad(background_shader);
});

app.raw_layer(1, |ctx| {
    // 3D content - raw layer (no input handling)
    ctx.set_camera(camera);
    ctx.draw_mesh(cube_mesh, transform);
});

app.ui_layer(2, LayerOptions::default().with_input(), |ui| {
    // UI layer with taffy layout (handles input)
    ui.flex_column(|ui| {
        ui.label("Score: 100");
        if ui.button("Reset") {
            // Handle click
        }
    });
});

app.raw_layer(3, |ctx| {
    // More 3D content on top (no input handling)
    ctx.draw_particles(particle_system);
});

app.ui_layer(4, LayerOptions::default().with_input(), |ui| {
    // Another UI layer (e.g., debug overlay)
    ui.window("Debug", |ui| {
        ui.label("FPS: 60");
        ui.checkbox("Show wireframe", &mut show_wireframe);
    });
});

// Modal example - just use higher z-order
if show_modal {
    app.ui_layer(10, LayerOptions::default().with_input(), |ui| {
        // Fullscreen backdrop blocks input to lower layers
        ui.fill_rect(ui.screen_bounds(), Color::rgba(0, 0, 0, 0.5));

        ui.centered(|ui| {
            ui.dialog(|ui| {
                ui.label("Are you sure?");
                if ui.button("Yes") || ui.button("No") {
                    show_modal = false;
                }
            });
        });
    });
}
```

### ID System

- Use hash of widget position in code + user data
- Allows tracking state across frames
- Similar to Dear ImGui's approach

### Entity System

Provides persistent state for widgets that need it (text fields, scroll areas, etc.) while keeping most UI stateless.

```rust
// Entity type - a reference-counted handle to persistent state
pub struct Entity<T> {
    id: EntityId,
    _phantom: PhantomData<T>,
}

// Creating entities for stateful widgets
let text_field = cx.new_entity(|_| TextFieldState {
    content: String::new(),
    cursor: 0,
    selection: None,
});

// Using entities
ui.text_field(text_field);

// One-off stateful widgets
ui.stateful(|ui, state: &mut ScrollState| {
    ui.scroll_area(|ui| {
        // Scrollable content
    });
});

// Entity cleanup happens automatically when no longer referenced
```

Benefits:

- Most UI remains stateless and simple
- Explicit about what has state
- Type-safe access to state
- Automatic memory management
- Can observe state changes

## Dependencies

- `glam` - Math library for 2D/3D operations ✅
- `taffy` - Flexbox/CSS Grid layout engine ✅ (not used yet)
- `metal` - Metal API bindings ✅
- `objc` - Objective-C runtime access ✅
- `cocoa` - macOS framework bindings (minimal use) ✅
- `core-foundation` - CF types ✅
- `core-graphics` - CG types ✅
- `core-text` - Text rendering ✅
- `palette` - Color space handling ✅
- `font-kit` - Font loading and rasterization ✅

## References

- gpui's metal renderer and platform layer
- Dear ImGui's immediate mode concepts
- Unity's IMGUI system
- Gio's approach to GPU-accelerated UI

## Open Questions

1. ~~How to handle retained state that must persist (e.g., text input cursor position)?~~ **Decided**: Entity system for explicit persistent state
2. Should UI layers support custom shaders or just use a standard UI shader?
3. How to efficiently batch draw calls within and across layers?
4. Should raw layers be able to read from previous layers (for effects)?
5. ~~How to handle input routing between layers?~~ **Decided**: Z-order with opt-in
6. Should we cache taffy trees between frames when nothing changes?
7. Can UI layers be transformed in 3D space as a whole?
8. When we add input to raw layers, how should 3D hit-testing work?
9. Should layer indices be integers or floats for finer control?
10. ~~Coordinate system design?~~ **Decided**: Explicit unit types (LogicalPixel, PhysicalPixel, WorldUnit, LocalUnit)

## Next Steps - Concrete Implementation Tasks

### Phase 0: Basic Window & Rendering ✅

See [window_and_rendering.md](window_and_rendering.md) for detailed implementation tracking.

**Completed**:

- ✅ Basic window creation with Metal layer
- ✅ Metal device and command queue setup
- ✅ Metal rendering pipeline with shaders
- ✅ Immediate mode UI context
- ✅ Quad rendering for UI elements
- ✅ Basic UI elements (group, rect, layout helpers)
- ✅ Color system with palette crate

**Current Status**: Core UI framework functional with widgets, input handling, and performance optimizations

### Completed Recently:
- Keyboard shortcuts system with global/contextual scopes
- Undo/redo system using Command pattern
- Vertex buffer pooling for Metal renderer
- Text measurement caching
- Button and Checkbox widgets with focus support
- Scroll container with momentum scrolling

### Remaining Work:
- Input field and slider widgets
- 3D features (transformations, camera, mesh rendering)
- Animations and transitions
- Advanced layouts (grid)
- Documentation improvements
