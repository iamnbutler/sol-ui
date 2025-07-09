use crate::metal_renderer::MetalRenderer;
use crate::taffy::UiTaffyContext;
use crate::ui::UiContext;
use glam::Vec2;
use metal::CommandBufferRef;
use std::any::Any;

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
    /// Get the z-index of this layer (higher values render on top)
    fn z_index(&self) -> i32;

    /// Get the options for this layer
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
    );

    /// Handle input if this layer is configured to receive it
    fn handle_input(&mut self, _event: &InputEvent) -> bool {
        false
    }

    /// Get mutable access as Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// A raw layer with direct shader access
pub struct RawLayer<F> {
    z_index: i32,
    options: LayerOptions,
    render_fn: F,
}

impl<F> RawLayer<F>
where
    F: FnMut(&mut RawLayerContext),
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
    pub size: Vec2,
}

impl<'a> RawLayerContext<'a> {
    /// Draw a fullscreen quad with a custom shader
    pub fn draw_fullscreen_quad(&mut self, _shader: ()) {
        // TODO: Implement custom shader support
        todo!("Custom shader support not yet implemented")
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
    F: FnMut(&mut RawLayerContext) + Any,
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
        _drawable: &metal::MetalDrawableRef,
        size: Vec2,
        _scale_factor: f32,
        _text_system: &mut crate::text_system::TextSystem,
        _is_first_layer: bool,
    ) {
        let mut ctx = RawLayerContext {
            renderer,
            command_buffer,
            size,
        };
        (self.render_fn)(&mut ctx);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A UI layer using the immediate mode API
pub struct UiLayer<F> {
    z_index: i32,
    options: LayerOptions,
    render_fn: F,
}

impl<F> UiLayer<F>
where
    F: FnMut(&mut UiContext),
{
    pub fn new(z_index: i32, options: LayerOptions, render_fn: F) -> Self {
        Self {
            z_index,
            options,
            render_fn,
        }
    }
}

impl<F> Layer for UiLayer<F>
where
    F: FnMut(&mut UiContext) + Any,
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
        scale_factor: f32,
        text_system: &mut crate::text_system::TextSystem,
        is_first_layer: bool,
    ) {
        // Create a new UI context for this layer
        let mut ui_context = UiContext::new(size);

        // Run the UI function
        (self.render_fn)(&mut ui_context);

        // Render the UI draw list
        let draw_list = ui_context.draw_list();
        // Determine load action and clear color based on layer options
        let (load_action, clear_color) = if self.options.clear || is_first_layer {
            (metal::MTLLoadAction::Clear, self.options.clear_color)
        } else {
            (
                metal::MTLLoadAction::Load,
                metal::MTLClearColor::new(0.0, 0.0, 0.0, 0.0),
            )
        };

        renderer.render_draw_list(
            draw_list,
            command_buffer,
            drawable,
            (size.x, size.y),
            scale_factor,
            text_system,
            load_action,
            clear_color,
        );
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// A UI layer that uses Taffy for layout
pub struct TaffyUiLayer<F> {
    options: LayerOptions,
    render_fn: F,
}

impl<F> TaffyUiLayer<F>
where
    F: FnMut(&mut UiTaffyContext) + 'static,
{
    /// Create a new Taffy UI layer
    pub fn new(options: LayerOptions, render_fn: F) -> Self {
        Self { options, render_fn }
    }
}

impl<F> Layer for TaffyUiLayer<F>
where
    F: FnMut(&mut UiTaffyContext) + 'static,
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
    ) {
        // Create a new Taffy UI context
        let mut ui_context = UiTaffyContext::new(size, scale_factor);

        // Execute the render function to build the UI tree
        (self.render_fn)(&mut ui_context);

        // Compute layout using Taffy with text measurement
        if let Err(e) = ui_context.compute_layout(text_system) {
            eprintln!("Failed to compute layout: {:?}", e);
            return;
        }

        // Build draw commands from the laid out tree
        let draw_list = match ui_context.build_draw_list(text_system) {
            Ok(list) => list,
            Err(e) => {
                eprintln!("Failed to build draw list: {:?}", e);
                return;
            }
        };

        // Determine load action and clear color
        let (load_action, clear_color) = if is_first_layer {
            (
                metal::MTLLoadAction::Clear,
                metal::MTLClearColor::new(0.95, 0.95, 0.95, 1.0), // Light gray background
            )
        } else {
            (
                metal::MTLLoadAction::Load,
                metal::MTLClearColor::new(0.0, 0.0, 0.0, 0.0),
            )
        };

        // Render the draw list
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
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
        F: FnMut(&mut RawLayerContext) + Any + 'static,
    {
        let layer = RawLayer::new(z_index, options, render_fn);
        self.add_layer(Box::new(layer));
    }

    /// Add a UI layer
    pub fn add_ui_layer<F>(&mut self, z_index: i32, options: LayerOptions, render_fn: F)
    where
        F: FnMut(&mut UiContext) + Any + 'static,
    {
        let layer = UiLayer::new(z_index, options, render_fn);
        self.add_layer(Box::new(layer));
    }

    /// Add a Taffy UI layer
    pub fn add_taffy_ui_layer<F>(&mut self, z_index: i32, options: LayerOptions, render_fn: F)
    where
        F: FnMut(&mut UiTaffyContext) + Any + 'static,
    {
        let layer = TaffyUiLayer::new(options.with_z_index(z_index), render_fn);
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

    /// Render all layers in order
    pub fn render(
        &mut self,
        renderer: &mut MetalRenderer,
        command_buffer: &CommandBufferRef,
        drawable: &metal::MetalDrawableRef,
        size: Vec2,
        text_system: &mut crate::text_system::TextSystem,
        scale_factor: f32,
    ) {
        for (i, (_, layer)) in self.layers.iter_mut().enumerate() {
            let is_first_layer = i == 0;
            layer.render(
                renderer,
                command_buffer,
                drawable,
                size,
                scale_factor,
                text_system,
                is_first_layer,
            );
        }
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

/// Input event placeholder
#[derive(Debug, Clone)]
pub enum InputEvent {
    // TODO: Define input events
    MouseMove { position: Vec2 },
    MouseDown { position: Vec2, button: MouseButton },
    MouseUp { position: Vec2, button: MouseButton },
    KeyDown { key: Key },
    KeyUp { key: Key },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    // TODO: Define key codes
    A,
    B,
    C, // ... etc
}

// Re-export commonly used types
