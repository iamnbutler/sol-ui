//! Text input element with cursor, selection, and keyboard handling

use crate::{
    color::{colors, Color},
    element::{Element, LayoutContext, PaintContext},
    entity::{new_entity, read_entity, update_entity, Entity},
    geometry::{Corners, Edges, Rect},
    interaction::{
        registry::{get_element_state, register_element},
        ElementId, EventHandlers,
    },
    layer::Key,
    render::{PaintQuad, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Focus ring color for text inputs
const FOCUS_RING_COLOR: Color = colors::BLUE_400;
/// Focus ring width
const FOCUS_RING_WIDTH: f32 = 2.0;
/// Focus ring offset from element bounds
const FOCUS_RING_OFFSET: f32 = 2.0;
/// Default text input height
const DEFAULT_HEIGHT: f32 = 36.0;
/// Default horizontal padding
const DEFAULT_PADDING_H: f32 = 12.0;
/// Default vertical padding
const DEFAULT_PADDING_V: f32 = 8.0;
/// Cursor blink interval (not implemented yet - always visible)
const _CURSOR_BLINK_MS: u32 = 530;
/// Cursor width
const CURSOR_WIDTH: f32 = 2.0;
/// Selection highlight color (light blue)
const SELECTION_COLOR: Color = Color::new(0.68, 0.81, 0.97, 1.0);

/// State for a text input, persisted via the Entity system
#[derive(Debug, Clone)]
pub struct TextInputState {
    /// The current text content
    pub text: String,
    /// Cursor position (character index)
    pub cursor: usize,
    /// Selection start position (None if no selection)
    pub selection_start: Option<usize>,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection_start: None,
        }
    }
}

impl TextInputState {
    /// Create a new text input state with initial text
    pub fn new(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self {
            text,
            cursor,
            selection_start: None,
        }
    }

    /// Get the current selection range (start, end) in ascending order
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            if start <= self.cursor {
                (start, self.cursor)
            } else {
                (self.cursor, start)
            }
        })
    }

    /// Check if there is an active selection
    pub fn has_selection(&self) -> bool {
        self.selection_start.is_some() && self.selection_start != Some(self.cursor)
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<&str> {
        self.selection_range().map(|(start, end)| &self.text[start..end])
    }

    /// Delete the selected text and return cursor to start of selection
    pub fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection_range() {
            self.text.replace_range(start..end, "");
            self.cursor = start;
            self.selection_start = None;
        }
    }

    /// Insert text at cursor position (replacing any selection)
    pub fn insert(&mut self, s: &str) {
        if self.has_selection() {
            self.delete_selection();
        }
        self.text.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    /// Insert a single character at cursor position
    pub fn insert_char(&mut self, c: char) {
        if self.has_selection() {
            self.delete_selection();
        }
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.has_selection() {
            self.delete_selection();
        } else if self.cursor > 0 {
            // Find the previous character boundary
            let prev_char_start = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.remove(prev_char_start);
            self.cursor = prev_char_start;
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete(&mut self) {
        if self.has_selection() {
            self.delete_selection();
        } else if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            // If there's a selection, collapse to start
            if let Some((start, _)) = self.selection_range() {
                self.cursor = start;
                self.selection_start = None;
                return;
            }
        }

        if self.cursor > 0 {
            // Find previous character boundary
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            // If there's a selection, collapse to end
            if let Some((_, end)) = self.selection_range() {
                self.cursor = end;
                self.selection_start = None;
                return;
            }
        }

        if self.cursor < self.text.len() {
            // Find next character boundary
            self.cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
        }
    }

    /// Move cursor to start of text
    pub fn move_to_start(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
        self.cursor = 0;
    }

    /// Move cursor to end of text
    pub fn move_to_end(&mut self, extend_selection: bool) {
        if extend_selection {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
        self.cursor = self.text.len();
    }

    /// Select all text
    pub fn select_all(&mut self) {
        self.selection_start = Some(0);
        self.cursor = self.text.len();
    }

    /// Clear selection without moving cursor
    pub fn clear_selection(&mut self) {
        self.selection_start = None;
    }
}

/// Create a new text input element
pub fn text_input() -> TextInput {
    TextInput::new()
}

/// Create a new text input element with initial value
pub fn text_input_with_value(value: impl Into<String>) -> TextInput {
    TextInput::new().value(value)
}

/// A single-line text input element
pub struct TextInput {
    /// Initial value (used to initialize state)
    initial_value: String,
    /// Placeholder text shown when empty
    placeholder: Option<String>,
    /// Text style for input text
    text_style: TextStyle,
    /// Placeholder text style
    placeholder_style: TextStyle,
    /// Background color
    background: Color,
    /// Border color
    border_color: Color,
    /// Border width
    border_width: f32,
    /// Corner radius
    corner_radius: f32,
    /// Horizontal padding
    padding_h: f32,
    /// Vertical padding
    padding_v: f32,
    /// Whether the input is disabled
    disabled: bool,
    /// Element ID for interaction
    element_id: ElementId,
    /// Event handlers
    handlers: Rc<RefCell<EventHandlers>>,
    /// Persisted state entity
    state: Option<Entity<TextInputState>>,
    /// On change callback (called when text changes)
    on_change: Option<Rc<RefCell<Box<dyn FnMut(&str)>>>>,
    /// On submit callback (called when Enter is pressed)
    on_submit: Option<Rc<RefCell<Box<dyn FnMut(&str)>>>>,
    /// Cached layout node ID
    node_id: Option<NodeId>,
}

impl TextInput {
    /// Create a new text input
    pub fn new() -> Self {
        Self {
            initial_value: String::new(),
            placeholder: None,
            text_style: TextStyle {
                color: colors::BLACK,
                size: 14.0,
            },
            placeholder_style: TextStyle {
                color: colors::GRAY_400,
                size: 14.0,
            },
            background: colors::WHITE,
            border_color: colors::GRAY_300,
            border_width: 1.0,
            corner_radius: 4.0,
            padding_h: DEFAULT_PADDING_H,
            padding_v: DEFAULT_PADDING_V,
            disabled: false,
            element_id: ElementId::auto(),
            handlers: Rc::new(RefCell::new(EventHandlers::new())),
            state: None,
            on_change: None,
            on_submit: None,
            node_id: None,
        }
    }

    /// Set the initial value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.initial_value = value.into();
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set text style
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_style.color = color;
        self
    }

    /// Set text size
    pub fn text_size(mut self, size: f32) -> Self {
        self.text_style.size = size;
        self.placeholder_style.size = size;
        self
    }

    /// Set placeholder style
    pub fn placeholder_style(mut self, style: TextStyle) -> Self {
        self.placeholder_style = style;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set border color
    pub fn border_color(mut self, color: Color) -> Self {
        self.border_color = color;
        self
    }

    /// Set border width
    pub fn border_width(mut self, width: f32) -> Self {
        self.border_width = width;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set horizontal padding
    pub fn padding_h(mut self, padding: f32) -> Self {
        self.padding_h = padding;
        self
    }

    /// Set vertical padding
    pub fn padding_v(mut self, padding: f32) -> Self {
        self.padding_v = padding;
        self
    }

    /// Set padding (both horizontal and vertical)
    pub fn padding(mut self, h: f32, v: f32) -> Self {
        self.padding_h = h;
        self.padding_v = v;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set a stable element ID
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Set the on_change callback (called when text content changes)
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.on_change = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Set the on_submit callback (called when Enter is pressed)
    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.on_submit = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Get the current text value
    pub fn current_value(&self) -> Option<String> {
        self.state.as_ref().and_then(|s| read_entity(s, |state| state.text.clone()))
    }

    /// Get the state entity for external control
    pub fn state_entity(&self) -> Option<Entity<TextInputState>> {
        self.state.clone()
    }

    /// Set up keyboard event handlers
    fn setup_handlers(&self) {
        let state = self.state.clone();
        let on_change = self.on_change.clone();
        let on_submit = self.on_submit.clone();

        self.handlers.borrow_mut().on_key_down = Some(Box::new(move |key, modifiers, character, is_repeat| {
            let Some(ref state_entity) = state else { return };

            match key {
                // Character input
                _ if character.is_some() && !modifiers.command_or_ctrl() => {
                    if let Some(c) = character {
                        // Skip control characters except for normal printable chars
                        if !c.is_control() {
                            update_entity(state_entity, |s| s.insert_char(c));
                            if let Some(ref cb) = on_change {
                                if let Some(text) = read_entity(state_entity, |s| s.text.clone()) {
                                    (cb.borrow_mut())(&text);
                                }
                            }
                        }
                    }
                }

                // Backspace
                Key::Backspace => {
                    update_entity(state_entity, |s| s.backspace());
                    if let Some(ref cb) = on_change {
                        if let Some(text) = read_entity(state_entity, |s| s.text.clone()) {
                            (cb.borrow_mut())(&text);
                        }
                    }
                }

                // Delete
                Key::Delete => {
                    update_entity(state_entity, |s| s.delete());
                    if let Some(ref cb) = on_change {
                        if let Some(text) = read_entity(state_entity, |s| s.text.clone()) {
                            (cb.borrow_mut())(&text);
                        }
                    }
                }

                // Arrow keys
                Key::Left => {
                    update_entity(state_entity, |s| s.move_left(modifiers.shift));
                }
                Key::Right => {
                    update_entity(state_entity, |s| s.move_right(modifiers.shift));
                }

                // Home/End
                Key::Home => {
                    update_entity(state_entity, |s| s.move_to_start(modifiers.shift));
                }
                Key::End => {
                    update_entity(state_entity, |s| s.move_to_end(modifiers.shift));
                }

                // Cmd+A - select all
                Key::A if modifiers.command_or_ctrl() => {
                    update_entity(state_entity, |s| s.select_all());
                }

                // Enter - submit
                Key::Return if !is_repeat => {
                    if let Some(ref cb) = on_submit {
                        if let Some(text) = read_entity(state_entity, |s| s.text.clone()) {
                            (cb.borrow_mut())(&text);
                        }
                    }
                }

                // Escape - clear selection
                Key::Escape => {
                    update_entity(state_entity, |s| s.clear_selection());
                }

                _ => {}
            }
        }));
    }

    /// Calculate cursor X position based on text up to cursor
    fn calculate_cursor_x(&self, text: &str, cursor: usize, ctx: &mut PaintContext) -> f32 {
        if cursor == 0 {
            return 0.0;
        }

        let text_before_cursor = &text[..cursor.min(text.len())];
        let size = ctx.text_system.measure_text(
            text_before_cursor,
            &crate::text_system::TextConfig {
                font_stack: parley::FontStack::from("system-ui"),
                size: self.text_style.size,
                weight: parley::FontWeight::NORMAL,
                color: self.text_style.color.clone(),
                line_height: 1.2,
            },
            None,
            ctx.scale_factor,
        );
        size.x
    }
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for TextInput {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Initialize state entity if not already done
        if self.state.is_none() {
            self.state = Some(new_entity(TextInputState::new(&self.initial_value)));
            self.setup_handlers();
        }

        // Create style for the input container
        let style = Style {
            size: Size {
                width: Dimension::percent(1.0), // Full width by default
                height: Dimension::length(DEFAULT_HEIGHT),
            },
            padding: taffy::Rect {
                left: LengthPercentage::length(self.padding_h),
                right: LengthPercentage::length(self.padding_h),
                top: LengthPercentage::length(self.padding_v),
                bottom: LengthPercentage::length(self.padding_v),
            },
            ..Default::default()
        };

        let node_id = ctx.request_layout(style);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        // Register for interaction if not disabled
        if !self.disabled {
            register_element(self.element_id, self.handlers.clone());
        }

        // Get interaction state
        let interaction_state = get_element_state(self.element_id).unwrap_or_default();

        // Get text state
        let (text, cursor, selection_start) = self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| {
                (state.text.clone(), state.cursor, state.selection_start)
            }))
            .unwrap_or_else(|| (String::new(), 0, None));

        // Paint focus ring if focused
        if interaction_state.is_focused && !self.disabled {
            let focus_bounds = Rect::from_pos_size(
                bounds.pos - Vec2::splat(FOCUS_RING_OFFSET),
                bounds.size + Vec2::splat(FOCUS_RING_OFFSET * 2.0),
            );
            ctx.paint_quad(PaintQuad {
                bounds: focus_bounds,
                fill: colors::TRANSPARENT,
                corner_radii: Corners::all(self.corner_radius + FOCUS_RING_OFFSET),
                border_widths: Edges::all(FOCUS_RING_WIDTH),
                border_color: FOCUS_RING_COLOR,
            });
        }

        // Paint background
        let bg_color = if self.disabled {
            colors::GRAY_100
        } else {
            self.background
        };

        let border_color = if interaction_state.is_focused && !self.disabled {
            colors::BLUE_400
        } else {
            self.border_color
        };

        ctx.paint_quad(PaintQuad {
            bounds,
            fill: bg_color,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::all(self.border_width),
            border_color,
        });

        // Calculate text area bounds (inside padding)
        let text_area = Rect::from_pos_size(
            bounds.pos + Vec2::new(self.padding_h, self.padding_v),
            bounds.size - Vec2::new(self.padding_h * 2.0, self.padding_v * 2.0),
        );

        // Push clip for text area
        ctx.draw_list.push_clip(text_area);

        // Calculate text Y position (vertically centered)
        let text_height = self.text_style.size * 1.2; // Approximate line height
        let text_y = text_area.pos.y + (text_area.size.y - text_height) / 2.0;

        // Paint selection highlight if there's a selection
        if interaction_state.is_focused && selection_start.is_some() {
            let sel_start = selection_start.unwrap();
            if sel_start != cursor {
                let (start, end) = if sel_start <= cursor {
                    (sel_start, cursor)
                } else {
                    (cursor, sel_start)
                };

                let start_x = self.calculate_cursor_x(&text, start, ctx);
                let end_x = self.calculate_cursor_x(&text, end, ctx);

                let selection_bounds = Rect::from_pos_size(
                    Vec2::new(text_area.pos.x + start_x, text_y),
                    Vec2::new(end_x - start_x, text_height),
                );

                ctx.paint_quad(PaintQuad::filled(selection_bounds, SELECTION_COLOR));
            }
        }

        // Paint text or placeholder
        if text.is_empty() {
            if let Some(ref placeholder) = self.placeholder {
                ctx.paint_text(PaintText {
                    position: Vec2::new(text_area.pos.x, text_y),
                    text: placeholder.clone(),
                    style: self.placeholder_style.clone(),
                });
            }
        } else {
            ctx.paint_text(PaintText {
                position: Vec2::new(text_area.pos.x, text_y),
                text: text.clone(),
                style: if self.disabled {
                    TextStyle {
                        color: colors::GRAY_400,
                        ..self.text_style.clone()
                    }
                } else {
                    self.text_style.clone()
                },
            });
        }

        // Paint cursor if focused
        if interaction_state.is_focused && !self.disabled {
            let cursor_x = text_area.pos.x + self.calculate_cursor_x(&text, cursor, ctx);
            let cursor_bounds = Rect::from_pos_size(
                Vec2::new(cursor_x, text_y),
                Vec2::new(CURSOR_WIDTH, text_height),
            );
            ctx.paint_quad(PaintQuad::filled(cursor_bounds, colors::BLACK));
        }

        // Pop clip
        ctx.draw_list.pop_clip();

        // Register as focusable for hit testing
        if !self.disabled {
            ctx.register_focusable(self.element_id, bounds, 0);
        }
    }
}
