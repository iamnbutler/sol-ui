use crate::layout::UiContext;
use crate::platform::mac::metal_renderer::MetalRenderer;
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

/// Context passed to UI layer render functions
pub struct UiLayerContext {
    /// The immediate mode UI context
    pub ui: ImmediateUiContext,
    /// The screen size
    pub size: Vec2,
    /// The display scale factor
    pub scale_factor: f32,
}

impl UiLayerContext {
    /// Create a new UI layer context
    pub fn new(size: Vec2, scale_factor: f32) -> Self {
        Self {
            ui: ImmediateUiContext::new(size),
            size,
            scale_factor,
        }
    }
}

/// A UI layer that provides immediate mode GUI context
pub struct UiLayer<F> {
    z_index: i32,
    options: LayerOptions,
    render_fn: F,
}

impl<F> UiLayer<F>
where
    F: FnMut(&mut UiLayerContext) + 'static,
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
    F: FnMut(&mut UiLayerContext) + 'static,
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
        // Create UI layer context
        let mut context = UiLayerContext::new(size, scale_factor);
        context.ui.begin_frame();

        // Execute the render function
        (self.render_fn)(&mut context);

        // Get the draw list
        let draw_list = context.ui.end_frame();

        // Determine load action and clear color
        let (load_action, clear_color) = if is_first_layer {
            (
                metal::MTLLoadAction::Clear,
                metal::MTLClearColor::new(0.1, 0.1, 0.1, 1.0),
            )
        } else {
            (
                metal::MTLLoadAction::Load,
                metal::MTLClearColor::new(0.0, 0.0, 0.0, 0.0),
            )
        };

        // Render the draw list
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
    F: Fn() -> Box<dyn crate::layout::Element> + 'static,
{
    /// Create a new Taffy UI layer
    pub fn new(options: LayerOptions, render_fn: F) -> Self {
        Self { options, render_fn }
    }
}

impl<F> Layer for TaffyUiLayer<F>
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
        // Build the UI tree by calling the render function
        let root_element = (self.render_fn)();

        // Create a UI context and build the tree
        let ui_context = UiContext::new(size, scale_factor).build(root_element);

        // Render the UI tree and get draw commands
        let draw_list = match ui_context.render(text_system) {
            Ok(list) => list,
            Err(e) => {
                eprintln!("Failed to render UI: {:?}", e);
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
        F: for<'a> FnMut(&mut RawLayerContext<'a>) + Any + 'static,
    {
        let layer = RawLayer::new(z_index, options, render_fn);
        self.add_layer(Box::new(layer));
    }

    /// Add a UI layer
    pub fn add_ui_layer<F>(&mut self, z_index: i32, options: LayerOptions, render_fn: F)
    where
        F: FnMut(&mut UiLayerContext) + Any + 'static,
    {
        let layer = UiLayer::new(z_index, options, render_fn);
        self.add_layer(Box::new(layer));
    }

    /// Add a Taffy UI layer
    pub fn add_taffy_ui_layer<F>(&mut self, z_index: i32, options: LayerOptions, render_fn: F)
    where
        F: Fn() -> Box<dyn crate::layout::Element> + Any + 'static,
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

// === Immediate Mode UI Context ===

use crate::color::{Color, ColorExt, colors::WHITE};
use crate::draw::{DrawCommand, DrawList, FrameStyle, TextStyle};
use crate::element::{ElementId, IdStack};
use crate::geometry::Rect;

/// The main context for immediate mode UI
pub struct ImmediateUiContext {
    /// Draw commands accumulated during the frame
    pub draw_list: DrawList,

    /// Widget ID stack for hierarchical ID generation
    id_stack: IdStack,

    /// Current cursor position for layout
    cursor: Vec2,

    /// Layout state
    layout: LayoutState,

    /// Window/screen dimensions
    screen_size: Vec2,
}

/// Layout state for automatic positioning
#[derive(Debug, Clone)]
struct LayoutState {
    /// Starting position of current layout group
    start_pos: Vec2,

    /// Maximum extent in the cross-axis direction
    max_cross_axis: f32,

    /// Current layout direction
    direction: LayoutDirection,

    /// Spacing between elements
    spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LayoutDirection {
    Vertical,
    Horizontal,
}

impl ImmediateUiContext {
    /// Create a new UI context with the given screen dimensions
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            draw_list: DrawList::with_viewport(Rect::from_pos_size(Vec2::ZERO, screen_size)),
            id_stack: IdStack::new(),
            cursor: Vec2::ZERO,
            layout: LayoutState {
                start_pos: Vec2::ZERO,
                max_cross_axis: 0.0,
                direction: LayoutDirection::Vertical,
                spacing: 5.0,
            },
            screen_size,
        }
    }

    /// Begin a new frame. Call this before any UI elements.
    pub fn begin_frame(&mut self) {
        self.draw_list.clear();
        // Re-apply viewport after clearing
        self.draw_list
            .set_viewport(Some(Rect::from_pos_size(Vec2::ZERO, self.screen_size)));
        self.cursor = Vec2::new(10.0, 10.0); // Default margin
        self.layout.start_pos = self.cursor;
        self.layout.max_cross_axis = 0.0;
        self.id_stack.reset_child_counter();
    }

    /// End the current frame and return the draw list
    pub fn end_frame(&mut self) -> &DrawList {
        &self.draw_list
    }

    /// Get the draw list without ending the frame (for layer system)
    pub fn draw_list(&self) -> &DrawList {
        &self.draw_list
    }

    /// Set the current cursor position for manual layout
    pub fn set_cursor(&mut self, pos: Vec2) {
        self.cursor = pos;
        self.layout.start_pos = pos;
    }

    /// Get the current cursor position
    pub fn cursor(&self) -> Vec2 {
        self.cursor
    }

    /// Get the screen dimensions
    pub fn screen_size(&self) -> Vec2 {
        self.screen_size
    }

    /// Update screen dimensions (e.g., on window resize)
    pub fn set_screen_size(&mut self, width: f32, height: f32) {
        self.screen_size = Vec2::new(width, height);
        // Update viewport for culling
        self.draw_list
            .set_viewport(Some(Rect::from_pos_size(Vec2::ZERO, self.screen_size)));
    }

    /// Draw text at the current cursor position
    pub fn text(&mut self, text: impl Into<String>) {
        self.text_styled(text, TextStyle::default());
    }

    /// Draw text with custom styling
    pub fn text_styled(&mut self, text: impl Into<String>, style: TextStyle) {
        let text = text.into();

        // Add text to draw list
        self.draw_list.add_text(self.cursor, &text, style.clone());

        // Advance cursor (approximate - we don't have proper text metrics yet)
        let text_height = style.size;
        let text_width = text.len() as f32 * style.size * 0.6; // Rough approximation

        self.advance_cursor(Vec2::new(text_width, text_height));
    }

    /// Begin a group (container) with optional background
    pub fn group<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.group_styled(None, f)
    }

    /// Begin a group with background color
    pub fn group_styled<R>(
        &mut self,
        background: Option<Color>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        // Save current layout state
        let saved_cursor = self.cursor;
        let saved_layout = self.layout.clone();

        // Generate ID for this group
        let id = self.next_id();
        self.id_stack.push(id);

        // Record starting position
        let start_pos = self.cursor;

        // Reset layout for group content
        self.layout.start_pos = self.cursor;
        self.layout.max_cross_axis = 0.0;

        // Record position before content (for background insertion)
        let draw_pos = self.draw_list.current_pos();

        // Call the group content function
        let result = f(self);

        // Calculate group bounds
        let size = match self.layout.direction {
            LayoutDirection::Vertical => {
                Vec2::new(self.layout.max_cross_axis, self.cursor.y - start_pos.y)
            }
            LayoutDirection::Horizontal => {
                Vec2::new(self.cursor.x - start_pos.x, self.layout.max_cross_axis)
            }
        };

        // Draw background if specified (insert at recorded position)
        if let Some(color) = background {
            self.draw_list
                .insert_rect_at(draw_pos, Rect::from_pos_size(start_pos, size), color);
        }

        // Restore layout state and advance cursor
        self.cursor = saved_cursor;
        self.layout = saved_layout;
        self.advance_cursor(size);

        // Pop ID
        self.id_stack.pop();

        result
    }

    /// Create a vertical layout group
    pub fn vertical<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let saved_direction = self.layout.direction;
        self.layout.direction = LayoutDirection::Vertical;
        let result = self.group(f);
        self.layout.direction = saved_direction;
        result
    }

    /// Create a horizontal layout group
    pub fn horizontal<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let saved_direction = self.layout.direction;
        self.layout.direction = LayoutDirection::Horizontal;
        let result = self.group(f);
        self.layout.direction = saved_direction;
        result
    }

    /// Add spacing in the current layout direction
    pub fn space(&mut self, amount: f32) {
        match self.layout.direction {
            LayoutDirection::Vertical => self.cursor.y += amount,
            LayoutDirection::Horizontal => self.cursor.x += amount,
        }
    }

    /// Draw a colored rectangle
    pub fn rect(&mut self, size: Vec2, color: Color) {
        let rect = Rect::from_pos_size(self.cursor, size);
        self.draw_list.add_rect(rect, color);
        self.advance_cursor(size);
    }

    /// Draw a frame with rounded corners and optional border
    pub fn frame(&mut self, size: Vec2, style: FrameStyle) {
        let rect = Rect::from_pos_size(self.cursor, size);
        self.draw_list.add_frame(rect, style);
        self.advance_cursor(size);
    }

    /// Create a frame container that can hold other UI elements
    pub fn frame_container<R>(&mut self, style: FrameStyle, f: impl FnOnce(&mut Self) -> R) -> R {
        // Save current layout state
        let saved_cursor = self.cursor;
        let saved_layout = self.layout.clone();

        // Generate ID for this frame
        let id = self.next_id();
        self.id_stack.push(id);

        // Record starting position
        let start_pos = self.cursor;

        // Reset layout for frame content
        self.layout.start_pos = self.cursor;
        self.layout.max_cross_axis = 0.0;

        // Draw the frame background first
        let temp_rect = Rect::from_pos_size(start_pos, Vec2::new(1.0, 1.0));
        self.draw_list.add_frame(temp_rect, style.clone());
        let frame_cmd_pos = self.draw_list.commands().len() - 1;

        // Push clipping for content
        self.draw_list.push_clip(temp_rect);
        let clip_cmd_pos = self.draw_list.commands().len() - 1;

        // Call the frame content function
        let result = f(self);

        // Pop clipping
        self.draw_list.pop_clip();

        // Calculate actual frame bounds
        let size = match self.layout.direction {
            LayoutDirection::Vertical => {
                Vec2::new(self.layout.max_cross_axis, self.cursor.y - start_pos.y)
            }
            LayoutDirection::Horizontal => {
                Vec2::new(self.cursor.x - start_pos.x, self.layout.max_cross_axis)
            }
        };

        // Update frame command with actual size
        let frame_rect = Rect::from_pos_size(start_pos, size);
        let commands = self.draw_list.commands_mut();
        if let Some(DrawCommand::Frame { rect, .. }) = commands.get_mut(frame_cmd_pos) {
            *rect = frame_rect;
        }

        // Update clip command with actual size
        if let Some(DrawCommand::PushClip { rect }) = commands.get_mut(clip_cmd_pos) {
            *rect = frame_rect;
        }

        // Restore layout state and advance cursor
        self.cursor = saved_cursor;
        self.layout = saved_layout;
        self.advance_cursor(size);

        // Pop ID
        self.id_stack.pop();

        result
    }

    /// Create a frame container with padding
    pub fn frame_container_padded<R>(
        &mut self,
        style: FrameStyle,
        padding: f32,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.frame_container(style, |ui| {
            // Apply padding
            ui.cursor += Vec2::new(padding, padding);
            ui.layout.start_pos = ui.cursor;

            let result = f(ui);

            // Add padding to final size
            ui.cursor += Vec2::new(padding, padding);

            result
        })
    }

    /// Advance cursor based on element size and current layout
    pub fn advance_cursor(&mut self, size: Vec2) {
        match self.layout.direction {
            LayoutDirection::Vertical => {
                self.cursor.y += size.y + self.layout.spacing;
                self.layout.max_cross_axis = self.layout.max_cross_axis.max(size.x);
            }
            LayoutDirection::Horizontal => {
                self.cursor.x += size.x + self.layout.spacing;
                self.layout.max_cross_axis = self.layout.max_cross_axis.max(size.y);
            }
        }
    }

    /// Generate the next element ID
    fn next_id(&self) -> ElementId {
        // For now, use a simple counter-based ID
        // In a real implementation, this would use the element_id! macro
        ElementId::from_source_location("ui", 0, 0)
    }
}

/// Builder-style methods for common patterns
impl ImmediateUiContext {
    /// Create a centered group
    pub fn centered<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let center = self.screen_size * 0.5;
        self.set_cursor(center);
        self.group(f)
    }

    /// Create a window-style container
    pub fn window<R>(
        &mut self,
        title: &str,
        pos: Vec2,
        size: Vec2,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.set_cursor(pos);

        // Window background
        self.draw_list
            .add_rect(Rect::from_pos_size(pos, size), Color::rgb(0.2, 0.2, 0.2));

        // Title bar
        let title_height = 24.0;
        self.draw_list.add_rect(
            Rect::from_pos_size(pos, Vec2::new(size.x, title_height)),
            Color::rgb(0.3, 0.3, 0.3),
        );

        // Title text
        self.set_cursor(pos + Vec2::new(8.0, 4.0));
        self.text_styled(
            title,
            TextStyle {
                size: 16.0,
                color: WHITE,
            },
        );

        // Content area
        self.set_cursor(pos + Vec2::new(8.0, title_height + 8.0));

        // Clip content to window bounds
        self.draw_list.push_clip(Rect::from_pos_size(
            pos + Vec2::new(0.0, title_height),
            Vec2::new(size.x, size.y - title_height),
        ));

        let result = f(self);

        self.draw_list.pop_clip();

        result
    }
}
