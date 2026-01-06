use crate::{
    element::{Element, LayoutContext},
    entity::{EntityStore, clear_entity_store, set_entity_store},
    interaction::{
        InteractionSystem,
        hit_test::HitTestBuilder,
        registry::{ElementRegistry, clear_current_registry, set_current_registry},
    },
    layout_engine::TaffyLayoutEngine,
    platform::mac::metal_renderer::MetalRenderer,
    render::{DrawList, PaintContext},
};
use glam::Vec2;
use metal::CommandBufferRef;
use std::any::Any;
use tracing::{debug, info, info_span};

/// Options for configuring a layer
#[derive(Debug, Clone)]
pub struct LayerOptions {
    /// Z-index for layer ordering
    pub z_index: i32,
    /// Whether this layer receives input events
    pub receives_input: bool,
    /// Blend mode for compositing
    pub blend_mode: BlendMode,
    /// Whether to clear before rendering
    pub clear: bool,
    /// Clear color (if clearing is enabled
    pub clear_color: metal::MTLClearColor,
}

impl Default for LayerOptions {
    fn default() -> Self {
        Self {
            z_index: 0,
            receives_input: false,
            blend_mode: BlendMode::Alpha,
            clear: false,
            clear_color: metal::MTLClearColor::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

impl LayerOptions {
    /// Create a new LayerOptions with the specified z_index
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }
}

impl LayerOptions {
    /// Enable input handling for this layer
    pub fn with_input(mut self) -> Self {
        self.receives_input = true;
        self
    }

    /// Set the blend mode
    pub fn with_blend_mode(mut self, mode: BlendMode) -> Self {
        self.blend_mode = mode;
        self
    }

    /// Enable clearing before rendering
    pub fn with_clear(mut self) -> Self {
        self.clear = true;
        self
    }

    /// Set the clear color
    pub fn with_clear_color(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.clear_color = metal::MTLClearColor::new(r, g, b, a);
        self
    }
}

/// Blend modes for layer compositing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Standard alpha blending (default)
    Alpha,
    /// Additive blending
    Additive,
    /// Multiply blending
    Multiply,
    /// Replace (no blending)
    Replace,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::Alpha
    }
}

/// Core trait that all layers must implement
pub trait Layer: Any {
    /// Get the z-index for layer ordering
    fn z_index(&self) -> i32;

    /// Get layer options
    fn options(&self) -> &LayerOptions;

    /// Render this layer
    fn render(
        &mut self,
        renderer: &mut MetalRenderer,
        command_buffer: &CommandBufferRef,
        drawable: &metal::MetalDrawableRef,
        size: Vec2,
        scale_factor: f32,
        text_system: &mut crate::text_system::TextSystem,
        is_first_layer: bool,
        animation_frame_requested: &mut bool,
        elapsed_time: f32,
    );

    /// Handle input events
    fn handle_input(&mut self, _event: &InputEvent) -> bool {
        false
    }

    /// Get mutable reference as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Invalidate any cached data, forcing a rebuild on next render
    fn invalidate(&mut self) {
        // Default implementation does nothing
    }
}

/// A raw layer with direct shader access
pub struct RawLayer<F> {
    z_index: i32,
    options: LayerOptions,
    render_fn: F,
}

impl<F> RawLayer<F>
where
    F: for<'a> FnMut(&mut RawLayerContext<'a>) + Any,
{
    pub fn new(z_index: i32, options: LayerOptions, render_fn: F) -> Self {
        Self {
            z_index,
            options,
            render_fn,
        }
    }
}

/// Context provided to raw layer render functions
pub struct RawLayerContext<'a> {
    pub renderer: &'a mut MetalRenderer,
    pub command_buffer: &'a CommandBufferRef,
    pub drawable: &'a metal::MetalDrawableRef,
    pub size: Vec2,
    pub time: f32,
    animation_frame_requested: &'a mut bool,
}

impl<'a> RawLayerContext<'a> {
    /// Request that another frame be rendered immediately after this one
    pub fn request_animation_frame(&mut self) {
        *self.animation_frame_requested = true;
    }

    /// Draw a fullscreen quad with a custom shader
    pub fn draw_fullscreen_quad(&mut self, shader_source: &str) {
        self.renderer.draw_fullscreen_quad(
            self.command_buffer,
            self.drawable,
            shader_source,
            self.size,
            self.time,
        );
    }

    /// Set camera for 3D rendering
    pub fn set_camera(&mut self, _camera: ()) {
        // TODO: Implement camera system
        todo!("Camera system not yet implemented")
    }

    /// Draw a 3D mesh
    pub fn draw_mesh(&mut self, _mesh: (), _transform: ()) {
        // TODO: Implement mesh rendering
        todo!("Mesh rendering not yet implemented")
    }
}

impl<F> Layer for RawLayer<F>
where
    F: for<'a> FnMut(&mut RawLayerContext<'a>) + Any,
{
    fn z_index(&self) -> i32 {
        self.z_index
    }

    fn options(&self) -> &LayerOptions {
        &self.options
    }

    fn render(
        &mut self,
        renderer: &mut MetalRenderer,
        command_buffer: &CommandBufferRef,
        drawable: &metal::MetalDrawableRef,
        size: Vec2,
        _scale_factor: f32,
        _text_system: &mut crate::text_system::TextSystem,
        _is_first_layer: bool,
        animation_frame_requested: &mut bool,
        elapsed_time: f32,
    ) {
        let _raw_render_span = info_span!("raw_layer_render").entered();

        let mut ctx = RawLayerContext {
            renderer,
            command_buffer,
            drawable,
            size,
            time: elapsed_time,
            animation_frame_requested,
        };

        (self.render_fn)(&mut ctx);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn invalidate(&mut self) {
        // Raw layers don't cache anything, so nothing to invalidate
    }
}

/// A UI layer that uses Taffy for layout
pub struct UiLayer<F> {
    options: LayerOptions,
    render_fn: F,
    layout_engine: TaffyLayoutEngine,
    root_element: Option<Box<dyn Element>>,
    interaction_system: InteractionSystem,
    element_registry: std::rc::Rc<std::cell::RefCell<ElementRegistry>>,
    /// Tracks if the layer needs to rebuild its element tree
    needs_rebuild: bool,
    /// Last viewport size used for layout
    last_size: Option<Vec2>,
}

impl<F> UiLayer<F>
where
    F: Fn() -> Box<dyn Element> + 'static,
{
    /// Create a new Taffy UI layer
    pub fn new(options: LayerOptions, render_fn: F) -> Self {
        Self {
            options,
            render_fn,
            layout_engine: TaffyLayoutEngine::new(),
            root_element: None,
            interaction_system: InteractionSystem::new(),
            element_registry: std::rc::Rc::new(std::cell::RefCell::new(ElementRegistry::new())),
            needs_rebuild: true, // Always rebuild on first frame
            last_size: None,
        }
    }
}

impl<F> Layer for UiLayer<F>
where
    F: Fn() -> Box<dyn Element> + 'static,
{
    fn z_index(&self) -> i32 {
        self.options.z_index
    }

    fn options(&self) -> &LayerOptions {
        &self.options
    }

    fn render(
        &mut self,
        renderer: &mut MetalRenderer,
        command_buffer: &CommandBufferRef,
        drawable: &metal::MetalDrawableRef,
        size: Vec2,
        scale_factor: f32,
        text_system: &mut crate::text_system::TextSystem,
        is_first_layer: bool,
        _animation_frame_requested: &mut bool,
        _elapsed_time: f32,
    ) {
        let _render_span = info_span!("taffy_ui_layer_render").entered();

        // Track if size changed (useful for debugging and future optimizations)
        let size_changed = self.last_size != Some(size);
        if size_changed {
            self.last_size = Some(size);
        }

        // Currently we rebuild every frame (immediate mode pattern).
        // The needs_rebuild flag and size tracking are in place for future optimizations.
        // When needs_rebuild is false and size unchanged, we could potentially skip
        // layout recomputation, but this requires state change detection.
        self.needs_rebuild = false;

        // Begin new frame - prepares cache but doesn't clear retained nodes
        self.layout_engine.begin_frame();

        // Create root element
        self.root_element = Some((self.render_fn)());

        // Phase 1: Layout
        let layout_start = std::time::Instant::now();
        let mut layout_ctx = LayoutContext {
            engine: &mut self.layout_engine,
            text_system,
            scale_factor,
        };

        let root_node = self.root_element.as_mut().unwrap().layout(&mut layout_ctx);

        // Compute layout with screen size
        self.layout_engine
            .compute_layout(
                root_node,
                taffy::Size {
                    width: taffy::AvailableSpace::Definite(size.x),
                    height: taffy::AvailableSpace::Definite(size.y),
                },
                text_system,
                scale_factor,
            )
            .expect("Layout computation failed");

        // End frame - clean up nodes that weren't used
        self.layout_engine.end_frame();

        info!("Layout phase took {:?}", layout_start.elapsed());

        // Phase 2: Paint
        let mut draw_list =
            DrawList::with_viewport(crate::geometry::Rect::from_pos_size(Vec2::ZERO, size));

        // Clear and set the current element registry for this paint phase
        self.element_registry.borrow_mut().clear();
        set_current_registry(self.element_registry.clone());

        // Create hit test builder for this layer
        let hit_test_builder = std::rc::Rc::new(std::cell::RefCell::new(HitTestBuilder::new(
            0,
            self.z_index(),
        )));
        let mut paint_ctx = PaintContext {
            draw_list: &mut draw_list,
            text_system,
            layout_engine: &self.layout_engine,
            scale_factor,
            parent_offset: Vec2::ZERO,
            hit_test_builder: Some(hit_test_builder.clone()),
        };

        // Paint the root element (which will recursively paint children)
        let root_bounds = self.layout_engine.layout_bounds(root_node);
        self.root_element
            .as_mut()
            .unwrap()
            .paint(root_bounds, &mut paint_ctx);

        // Update hit test results in interaction system
        let hit_test_entries = hit_test_builder.borrow_mut().build();
        self.interaction_system.update_hit_test(hit_test_entries);

        // Clear the current registry after painting
        clear_current_registry();

        // Determine load action and clear color
        let (load_action, clear_color) = if is_first_layer {
            (
                metal::MTLLoadAction::Clear,
                metal::MTLClearColor::new(0.95, 0.95, 0.95, 1.0),
            )
        } else {
            (
                metal::MTLLoadAction::Load,
                metal::MTLClearColor::new(0.0, 0.0, 0.0, 0.0),
            )
        };

        // Render to screen
        renderer.render_draw_list(
            &draw_list,
            command_buffer,
            drawable,
            (size.x, size.y),
            scale_factor,
            text_system,
            load_action,
            clear_color,
        );
    }

    fn handle_input(&mut self, event: &InputEvent) -> bool {
        if !self.options.receives_input {
            return false;
        }

        // Process the event through the interaction system
        let interaction_events = self.interaction_system.handle_input(event);

        // Dispatch events to registered elements
        let mut handled = false;
        for event in &interaction_events {
            if self.element_registry.borrow_mut().dispatch_event(event) {
                handled = true;
            }
        }

        // Return true if any events were handled
        handled || !interaction_events.is_empty()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn invalidate(&mut self) {
        self.needs_rebuild = true;
    }
}

/// Manages all layers and handles rendering order
pub struct LayerManager {
    pub layers: Vec<(i32, Box<dyn Layer>)>,
}

impl LayerManager {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add a raw layer
    pub fn add_raw_layer<F>(&mut self, z_index: i32, options: LayerOptions, render_fn: F)
    where
        F: for<'a> FnMut(&mut RawLayerContext<'a>) + Any + 'static,
    {
        let layer = RawLayer::new(z_index, options, render_fn);
        self.add_layer(Box::new(layer));
    }

    /// Add a UI layer
    pub fn add_ui_layer<F>(&mut self, z_index: i32, options: LayerOptions, render_fn: F)
    where
        F: Fn() -> Box<dyn Element> + Any + 'static,
    {
        let layer = UiLayer::new(options.with_z_index(z_index), render_fn);
        self.add_layer(Box::new(layer));
    }

    /// Add a layer and maintain z-order
    fn add_layer(&mut self, layer: Box<dyn Layer>) {
        let z_index = layer.z_index();
        self.layers.push((z_index, layer));
        // Sort by z-index (ascending, so higher values render on top)
        self.layers.sort_by_key(|(z, _)| *z);
    }

    /// Clear all layers
    pub fn clear(&mut self) {
        self.layers.clear();
    }

    /// Invalidate all layers, forcing them to rebuild their cached data
    pub fn invalidate_all(&mut self) {
        debug!("Invalidating all layers");
        for (_, layer) in &mut self.layers {
            layer.invalidate();
        }
    }

    /// Invalidate a specific layer by z-index
    pub fn invalidate_layer(&mut self, z_index: i32) {
        if let Some((_, layer)) = self.layers.iter_mut().find(|(_, l)| l.z_index() == z_index) {
            debug!("Invalidating layer with z-index {}", z_index);
            layer.invalidate();
        }
    }

    /// Render all layers
    pub fn render(
        &mut self,
        renderer: &mut MetalRenderer,
        command_buffer: &CommandBufferRef,
        drawable: &metal::MetalDrawableRef,
        size: Vec2,
        text_system: &mut crate::text_system::TextSystem,
        entity_store: &mut EntityStore,
        scale_factor: f32,
        elapsed_time: f32,
    ) -> bool {
        let _render_all_span =
            info_span!("layer_manager_render_all", layer_count = self.layers.len()).entered();
        debug!("Rendering {} layers", self.layers.len());

        // Set thread-local entity store for this render frame
        set_entity_store(entity_store);

        let mut animation_frame_requested = false;

        for (i, (_, layer)) in self.layers.iter_mut().enumerate() {
            let _layer_span =
                info_span!("render_layer", layer_index = i, z_index = layer.z_index()).entered();
            let is_first_layer = i == 0;
            layer.render(
                renderer,
                command_buffer,
                drawable,
                size,
                scale_factor,
                text_system,
                is_first_layer,
                &mut animation_frame_requested,
                elapsed_time,
            );
        }

        // Clear thread-local and cleanup entities at frame boundary
        // cleanup() returns true if any observed entity was mutated
        clear_entity_store();
        let needs_reactive_render = entity_store.cleanup();

        // Request animation frame if explicitly requested OR if reactive state changed
        animation_frame_requested || needs_reactive_render
    }

    /// Handle input, starting from the topmost layer that accepts input
    pub fn handle_input(&mut self, event: &InputEvent) -> bool {
        // Iterate in reverse order (topmost layers first)
        for (_, layer) in self.layers.iter_mut().rev() {
            if layer.options().receives_input && layer.handle_input(event) {
                return true; // Event was consumed
            }
        }
        false
    }
}

/// Input events from the platform layer
#[derive(Debug, Clone)]
pub enum InputEvent {
    // Window events
    /// Window was resized - metal layer drawable size already updated
    WindowResize { size: Vec2 },

    // Mouse events
    MouseMove { position: Vec2 },
    MouseDown { position: Vec2, button: MouseButton },
    MouseUp { position: Vec2, button: MouseButton },
    MouseLeave,
    /// Scroll wheel event (positive delta = scroll up/left, negative = scroll down/right)
    ScrollWheel { position: Vec2, delta: Vec2 },

    // Keyboard events
    KeyDown {
        key: Key,
        modifiers: Modifiers,
        /// The character produced by this key press (if any)
        character: Option<char>,
        /// Whether this is a key repeat event
        is_repeat: bool,
    },
    KeyUp {
        key: Key,
        modifiers: Modifiers,
    },
    /// Modifier keys changed (shift, ctrl, cmd, alt)
    ModifiersChanged {
        modifiers: Modifiers,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Modifier key state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,   // Option key on macOS
    pub cmd: bool,   // Command key on macOS
    pub caps_lock: bool,
}

impl Modifiers {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns true if no modifiers are pressed
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.cmd
    }

    /// Returns true if only shift is pressed
    pub fn shift_only(&self) -> bool {
        self.shift && !self.ctrl && !self.alt && !self.cmd
    }

    /// Returns true if command (or ctrl on non-mac) is pressed
    pub fn command_or_ctrl(&self) -> bool {
        self.cmd || self.ctrl
    }
}

/// Virtual key codes matching macOS key codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Modifiers (for tracking purposes)
    Shift, Control, Alt, Command, CapsLock,
    LeftShift, RightShift, LeftControl, RightControl,
    LeftAlt, RightAlt, LeftCommand, RightCommand,

    // Navigation
    Up, Down, Left, Right,
    Home, End, PageUp, PageDown,

    // Editing
    Backspace, Delete, Tab, Return, Escape, Space,

    // Punctuation and symbols
    Minus, Equal, LeftBracket, RightBracket, Backslash,
    Semicolon, Quote, Grave, Comma, Period, Slash,

    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadDecimal, NumpadMultiply, NumpadPlus,
    NumpadClear, NumpadDivide, NumpadEnter, NumpadMinus, NumpadEquals,

    // Other
    Insert, PrintScreen, ScrollLock, Pause,

    /// Unknown key with raw key code
    Unknown(u16),
}

impl Key {
    /// Convert from macOS virtual key code
    pub fn from_keycode(code: u16) -> Self {
        match code {
            0x00 => Key::A,
            0x01 => Key::S,
            0x02 => Key::D,
            0x03 => Key::F,
            0x04 => Key::H,
            0x05 => Key::G,
            0x06 => Key::Z,
            0x07 => Key::X,
            0x08 => Key::C,
            0x09 => Key::V,
            0x0B => Key::B,
            0x0C => Key::Q,
            0x0D => Key::W,
            0x0E => Key::E,
            0x0F => Key::R,
            0x10 => Key::Y,
            0x11 => Key::T,
            0x12 => Key::Key1,
            0x13 => Key::Key2,
            0x14 => Key::Key3,
            0x15 => Key::Key4,
            0x16 => Key::Key6,
            0x17 => Key::Key5,
            0x18 => Key::Equal,
            0x19 => Key::Key9,
            0x1A => Key::Key7,
            0x1B => Key::Minus,
            0x1C => Key::Key8,
            0x1D => Key::Key0,
            0x1E => Key::RightBracket,
            0x1F => Key::O,
            0x20 => Key::U,
            0x21 => Key::LeftBracket,
            0x22 => Key::I,
            0x23 => Key::P,
            0x24 => Key::Return,
            0x25 => Key::L,
            0x26 => Key::J,
            0x27 => Key::Quote,
            0x28 => Key::K,
            0x29 => Key::Semicolon,
            0x2A => Key::Backslash,
            0x2B => Key::Comma,
            0x2C => Key::Slash,
            0x2D => Key::N,
            0x2E => Key::M,
            0x2F => Key::Period,
            0x30 => Key::Tab,
            0x31 => Key::Space,
            0x32 => Key::Grave,
            0x33 => Key::Backspace,
            0x35 => Key::Escape,
            0x37 => Key::Command,
            0x38 => Key::LeftShift,
            0x39 => Key::CapsLock,
            0x3A => Key::LeftAlt,
            0x3B => Key::LeftControl,
            0x3C => Key::RightShift,
            0x3D => Key::RightAlt,
            0x3E => Key::RightControl,
            // Function keys
            0x7A => Key::F1,
            0x78 => Key::F2,
            0x63 => Key::F3,
            0x76 => Key::F4,
            0x60 => Key::F5,
            0x61 => Key::F6,
            0x62 => Key::F7,
            0x64 => Key::F8,
            0x65 => Key::F9,
            0x6D => Key::F10,
            0x67 => Key::F11,
            0x6F => Key::F12,
            // Arrow keys
            0x7B => Key::Left,
            0x7C => Key::Right,
            0x7D => Key::Down,
            0x7E => Key::Up,
            // Navigation
            0x73 => Key::Home,
            0x77 => Key::End,
            0x74 => Key::PageUp,
            0x79 => Key::PageDown,
            0x75 => Key::Delete,
            // Numpad
            0x52 => Key::Numpad0,
            0x53 => Key::Numpad1,
            0x54 => Key::Numpad2,
            0x55 => Key::Numpad3,
            0x56 => Key::Numpad4,
            0x57 => Key::Numpad5,
            0x58 => Key::Numpad6,
            0x59 => Key::Numpad7,
            0x5B => Key::Numpad8,
            0x5C => Key::Numpad9,
            0x41 => Key::NumpadDecimal,
            0x43 => Key::NumpadMultiply,
            0x45 => Key::NumpadPlus,
            0x47 => Key::NumpadClear,
            0x4B => Key::NumpadDivide,
            0x4C => Key::NumpadEnter,
            0x4E => Key::NumpadMinus,
            0x51 => Key::NumpadEquals,
            _ => Key::Unknown(code),
        }
    }

    /// Returns true if this is a printable character key
    pub fn is_printable(&self) -> bool {
        matches!(
            self,
            Key::A | Key::B | Key::C | Key::D | Key::E | Key::F | Key::G | Key::H |
            Key::I | Key::J | Key::K | Key::L | Key::M | Key::N | Key::O | Key::P |
            Key::Q | Key::R | Key::S | Key::T | Key::U | Key::V | Key::W | Key::X |
            Key::Y | Key::Z |
            Key::Key0 | Key::Key1 | Key::Key2 | Key::Key3 | Key::Key4 |
            Key::Key5 | Key::Key6 | Key::Key7 | Key::Key8 | Key::Key9 |
            Key::Space | Key::Minus | Key::Equal | Key::LeftBracket | Key::RightBracket |
            Key::Backslash | Key::Semicolon | Key::Quote | Key::Grave | Key::Comma |
            Key::Period | Key::Slash
        )
    }

    /// Returns true if this is a modifier key
    pub fn is_modifier(&self) -> bool {
        matches!(
            self,
            Key::Shift | Key::Control | Key::Alt | Key::Command | Key::CapsLock |
            Key::LeftShift | Key::RightShift | Key::LeftControl | Key::RightControl |
            Key::LeftAlt | Key::RightAlt | Key::LeftCommand | Key::RightCommand
        )
    }
}

// Re-export commonly used types
