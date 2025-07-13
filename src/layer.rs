use crate::{
    element::{Element, LayoutContext, PaintContext},
    interaction::{
        InteractionSystem,
        hit_test::HitTestBuilder,
        registry::{ElementRegistry, clear_current_registry, set_current_registry},
    },
    layout_engine::TaffyLayoutEngine,
    platform::mac::metal_renderer::MetalRenderer,
    render::DrawList,
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
        let total_start = std::time::Instant::now();

        // Clear layout engine every frame
        self.layout_engine.clear();

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
        info!(
            "Computing layout with size: {:?}, scale_factor: {}",
            size, scale_factor
        );
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

        info!("Layout phase took {:?}", layout_start.elapsed());

        // Phase 2: Paint
        let paint_start = std::time::Instant::now();
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
        info!(
            "Created hit test builder for layer with z_index: {}, size: {:?}, scale_factor: {}",
            self.z_index(),
            size,
            scale_factor
        );
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

        info!("Paint phase took {:?}", paint_start.elapsed());

        // Update hit test results in interaction system
        let hit_test_entries = hit_test_builder.borrow_mut().build();
        info!("Built {} hit test entries", hit_test_entries.len());
        for entry in &hit_test_entries {
            info!("Hit test entry: {:?}", entry);
        }
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

        info!("Total UiLayer render took {:?}", total_start.elapsed());
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
        scale_factor: f32,
        elapsed_time: f32,
    ) -> bool {
        let _render_all_span =
            info_span!("layer_manager_render_all", layer_count = self.layers.len()).entered();
        debug!("Rendering {} layers", self.layers.len());

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

        animation_frame_requested
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
    MouseLeave,
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
