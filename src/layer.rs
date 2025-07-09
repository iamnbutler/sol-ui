use crate::layout::UiContext;
use crate::platform::mac::metal_renderer::MetalRenderer;
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
        scale_factor: f32,
        text_system: &mut crate::text_system::TextSystem,
        is_first_layer: bool,
    ) {
        let _raw_render_span = info_span!("raw_layer_render").entered();

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

    fn invalidate(&mut self) {
        // Raw layers don't cache anything, so nothing to invalidate
    }
}

/// A UI layer that uses Taffy for layout
pub struct UiLayer<F> {
    options: LayerOptions,
    render_fn: F,
    cached_draw_list: Option<crate::draw::DrawList>,
    cached_size: Option<Vec2>,
    needs_rebuild: bool,
}

impl<F> UiLayer<F>
where
    F: Fn() -> Box<dyn crate::layout::Element> + 'static,
{
    /// Create a new Taffy UI layer
    pub fn new(options: LayerOptions, render_fn: F) -> Self {
        Self {
            options,
            render_fn,
            cached_draw_list: None,
            cached_size: None,
            needs_rebuild: true,
        }
    }
}

impl<F> Layer for UiLayer<F>
where
    F: Fn() -> Box<dyn crate::layout::Element> + 'static,
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
        let _taffy_render_span = info_span!("taffy_ui_layer_render").entered();
        let total_start = std::time::Instant::now();

        // Check if we need to rebuild the UI tree
        let size_changed = self.cached_size != Some(size);
        if size_changed {
            info!(
                "Size changed from {:?} to {:?}, marking for rebuild",
                self.cached_size, size
            );
            self.needs_rebuild = true;
            self.cached_size = Some(size);
        }

        // Use cached draw list if available and no rebuild needed
        if !self.needs_rebuild && self.cached_draw_list.is_some() {
            debug!("Using cached draw list");
            let cached_list = self.cached_draw_list.as_ref().unwrap();

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

            // Render the cached draw list
            let _metal_render_span = info_span!(
                "metal_render_draw_list",
                command_count = cached_list.commands().len()
            )
            .entered();
            renderer.render_draw_list(
                cached_list,
                command_buffer,
                drawable,
                (size.x, size.y),
                scale_factor,
                text_system,
                load_action,
                clear_color,
            );
            return;
        }

        info!("Rebuilding UI tree for size {:?}", size);

        // Build the UI tree by calling the render function
        let build_start = std::time::Instant::now();
        let root_element = {
            let _build_span = info_span!("build_ui_tree").entered();
            (self.render_fn)()
        };
        info!("UI tree build took {:?}", build_start.elapsed());

        // Create a UI context and build the tree
        let context_start = std::time::Instant::now();
        let ui_context = {
            let _context_span = info_span!("create_ui_context_and_build").entered();
            UiContext::new(size, scale_factor).build(root_element)
        };
        info!("UI context creation took {:?}", context_start.elapsed());

        // Render the UI tree and get draw commands
        let render_start = std::time::Instant::now();
        let draw_list = {
            let _render_tree_span = info_span!("render_ui_tree").entered();
            match ui_context.render(text_system) {
                Ok(list) => {
                    debug!(
                        "UI tree rendered successfully with {} commands",
                        list.commands().len()
                    );
                    list
                }
                Err(e) => {
                    eprintln!("Failed to render UI: {:?}", e);
                    return;
                }
            }
        };

        info!("UI tree render took {:?}", render_start.elapsed());

        // Cache the draw list
        let cache_start = std::time::Instant::now();
        self.cached_draw_list = Some(draw_list.clone());
        self.needs_rebuild = false;
        info!("Caching draw list took {:?}", cache_start.elapsed());

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
        let _metal_render_span = info_span!(
            "metal_render_draw_list",
            command_count = draw_list.commands().len()
        )
        .entered();
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

        info!("Total UiLayer render took {:?}", total_start.elapsed());
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn invalidate(&mut self) {
        debug!("Invalidating UiLayer cache");
        self.needs_rebuild = true;
        self.cached_draw_list = None;
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
        F: Fn() -> Box<dyn crate::layout::Element> + Any + 'static,
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
        scale_factor: f32,
    ) {
        let _render_all_span =
            info_span!("layer_manager_render_all", layer_count = self.layers.len()).entered();
        debug!("Rendering {} layers", self.layers.len());

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
