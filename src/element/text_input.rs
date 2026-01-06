//! Text input element with cursor, selection, and keyboard handling

use crate::{
    color::{Color, ColorExt, colors},
    element::{Element, LayoutContext},
    entity::{Entity, read_entity, update_entity},
    geometry::{Corners, Edges, Rect},
    interaction::{
        ElementId, Interactable, InteractiveElement,
        registry::get_element_state,
    },
    layer::Key,
    render::{PaintContext, PaintQuad, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// State persisted across frames for a text input
#[derive(Debug, Clone)]
pub struct TextInputState {
    /// Current text content
    pub text: String,
    /// Cursor position (character index)
    pub cursor: usize,
    /// Selection start (if selecting)
    pub selection_start: Option<usize>,
    /// Whether cursor is visible (for blinking)
    pub cursor_visible: bool,
    /// Frame counter for cursor blinking
    pub blink_counter: u32,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            selection_start: None,
            cursor_visible: true,
            blink_counter: 0,
        }
    }
}

impl TextInputState {
    /// Create state with initial text
    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        let cursor = text.len();
        Self {
            text,
            cursor,
            ..Default::default()
        }
    }

    /// Get the selection range (start, end) where start <= end
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|start| {
            if start <= self.cursor {
                (start, self.cursor)
            } else {
                (self.cursor, start)
            }
        })
    }

    /// Get the selected text, if any
    pub fn selected_text(&self) -> Option<&str> {
        self.selection_range()
            .map(|(start, end)| &self.text[start..end])
    }

    /// Delete the current selection and return the deleted text
    pub fn delete_selection(&mut self) -> Option<String> {
        if let Some((start, end)) = self.selection_range() {
            let deleted = self.text[start..end].to_string();
            self.text.replace_range(start..end, "");
            self.cursor = start;
            self.selection_start = None;
            Some(deleted)
        } else {
            None
        }
    }

    /// Insert text at cursor, replacing any selection
    pub fn insert(&mut self, s: &str) {
        self.delete_selection();
        self.text.insert_str(self.cursor, s);
        self.cursor += s.len();
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.delete_selection().is_some() {
            return;
        }
        if self.cursor > 0 {
            // Find the previous character boundary
            let prev = self.text[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.remove(prev);
            self.cursor = prev;
        }
    }

    /// Delete character at cursor (delete key)
    pub fn delete(&mut self) {
        if self.delete_selection().is_some() {
            return;
        }
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    /// Move cursor left
    pub fn move_left(&mut self, extend_selection: bool) {
        if !extend_selection {
            // If there's a selection and not extending, move to start of selection
            if let Some((start, _)) = self.selection_range() {
                self.cursor = start;
                self.selection_start = None;
                return;
            }
        }

        if self.cursor > 0 {
            if extend_selection && self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
            // Find previous character boundary
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            if !extend_selection {
                self.selection_start = None;
            }
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self, extend_selection: bool) {
        if !extend_selection {
            // If there's a selection and not extending, move to end of selection
            if let Some((_, end)) = self.selection_range() {
                self.cursor = end;
                self.selection_start = None;
                return;
            }
        }

        if self.cursor < self.text.len() {
            if extend_selection && self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
            // Find next character boundary
            self.cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
            if !extend_selection {
                self.selection_start = None;
            }
        }
    }

    /// Move cursor to start
    pub fn move_to_start(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }
        self.cursor = 0;
        if !extend_selection {
            self.selection_start = None;
        }
    }

    /// Move cursor to end
    pub fn move_to_end(&mut self, extend_selection: bool) {
        if extend_selection && self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }
        self.cursor = self.text.len();
        if !extend_selection {
            self.selection_start = None;
        }
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
pub fn text_input(state: Entity<TextInputState>) -> TextInput {
    TextInput::new(state)
}

/// A single-line text input element
pub struct TextInput {
    /// Entity handle for persistent state
    state: Entity<TextInputState>,
    /// Element ID for interaction
    element_id: ElementId,
    /// Width of the input (None = fit to content/parent)
    width: Option<f32>,
    /// Height of the input
    height: f32,
    /// Text style
    text_style: TextStyle,
    /// Placeholder text
    placeholder: Option<String>,
    /// Placeholder text color
    placeholder_color: Color,
    /// Background color
    background: Color,
    /// Border color
    border_color: Color,
    /// Border color when focused
    focus_border_color: Color,
    /// Border width
    border_width: f32,
    /// Corner radius
    corner_radius: f32,
    /// Horizontal padding
    padding_h: f32,
    /// Vertical padding
    padding_v: f32,
    /// Cursor color
    cursor_color: Color,
    /// Selection background color
    selection_color: Color,
    /// Whether the input is disabled
    disabled: bool,
    /// On change callback (called when text changes)
    on_change: Option<Rc<RefCell<Box<dyn FnMut(&str)>>>>,
    /// On submit callback (called on Enter key)
    on_submit: Option<Rc<RefCell<Box<dyn FnMut(&str)>>>>,
    /// Cached layout node
    node_id: Option<NodeId>,
}

impl TextInput {
    pub fn new(state: Entity<TextInputState>) -> Self {
        Self {
            state,
            element_id: ElementId::auto(),
            width: None,
            height: 36.0,
            text_style: TextStyle {
                color: colors::BLACK,
                size: 14.0,
                line_height: 1.2,
            },
            placeholder: None,
            placeholder_color: colors::GRAY_400,
            background: colors::WHITE,
            border_color: colors::GRAY_300,
            focus_border_color: colors::BLUE_500,
            border_width: 1.0,
            corner_radius: 4.0,
            padding_h: 12.0,
            padding_v: 8.0,
            cursor_color: colors::BLACK,
            selection_color: colors::BLUE_500.with_alpha(0.3),
            disabled: false,
            on_change: None,
            on_submit: None,
            node_id: None,
        }
    }

    /// Set a stable element ID
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Set the width (None = auto/full)
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
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
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    /// Set placeholder color
    pub fn placeholder_color(mut self, color: Color) -> Self {
        self.placeholder_color = color;
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

    /// Set border color when focused
    pub fn focus_border_color(mut self, color: Color) -> Self {
        self.focus_border_color = color;
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

    /// Set both paddings
    pub fn padding(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding_h = horizontal;
        self.padding_v = vertical;
        self
    }

    /// Set cursor color
    pub fn cursor_color(mut self, color: Color) -> Self {
        self.cursor_color = color;
        self
    }

    /// Set selection highlight color
    pub fn selection_color(mut self, color: Color) -> Self {
        self.selection_color = color;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the on_change callback
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.on_change = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Set the on_submit callback (Enter key)
    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&str) + 'static,
    {
        self.on_submit = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Get the element ID
    pub fn element_id(&self) -> ElementId {
        self.element_id
    }
}

impl Element for TextInput {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        let style = Style {
            size: Size {
                width: self.width
                    .map(Dimension::length)
                    .unwrap_or(Dimension::auto()),
                height: Dimension::length(self.height),
            },
            min_size: Size {
                width: Dimension::length(100.0), // Minimum width
                height: Dimension::auto(),
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

        // Get interaction state
        let interaction_state = get_element_state(self.element_id).unwrap_or_default();
        let is_focused = interaction_state.is_focused;

        // Read current state from entity
        let (text, cursor, selection_start, cursor_visible) = read_entity(&self.state, |s| {
            (s.text.clone(), s.cursor, s.selection_start, s.cursor_visible)
        }).unwrap_or_default();

        // Determine border color based on focus
        let current_border_color = if is_focused && !self.disabled {
            self.focus_border_color
        } else {
            self.border_color
        };

        // Paint background
        ctx.paint_quad(PaintQuad {
            bounds,
            fill: if self.disabled { colors::GRAY_100 } else { self.background },
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::all(self.border_width),
            border_color: current_border_color,
        });

        // Calculate text area
        let text_area = Rect::from_pos_size(
            bounds.pos + Vec2::new(self.padding_h, self.padding_v),
            bounds.size - Vec2::new(self.padding_h * 2.0, self.padding_v * 2.0),
        );

        // Determine what to display
        let display_text = if text.is_empty() {
            self.placeholder.as_deref().unwrap_or("")
        } else {
            &text
        };
        let is_placeholder = text.is_empty() && self.placeholder.is_some();

        // Measure text for cursor positioning
        let text_config = crate::text_system::TextConfig {
            font_stack: parley::FontStack::from("system-ui"),
            size: self.text_style.size,
            weight: parley::FontWeight::NORMAL,
            color: self.text_style.color.clone(),
            line_height: 1.2,
        };

        // Paint selection highlight if present
        if !is_placeholder && selection_start.is_some() {
            let (sel_start, sel_end) = if let Some(start) = selection_start {
                if start <= cursor {
                    (start, cursor)
                } else {
                    (cursor, start)
                }
            } else {
                (0, 0)
            };

            if sel_start != sel_end {
                // Measure text up to selection start and end
                let start_text = &text[..sel_start];
                let end_text = &text[..sel_end];

                let start_width = if start_text.is_empty() {
                    0.0
                } else {
                    ctx.text_system.measure_text(start_text, &text_config, None, ctx.scale_factor).x
                };
                let end_width = ctx.text_system.measure_text(end_text, &text_config, None, ctx.scale_factor).x;

                let selection_rect = Rect::from_pos_size(
                    Vec2::new(text_area.pos.x + start_width, text_area.pos.y),
                    Vec2::new(end_width - start_width, text_area.size.y),
                );
                ctx.paint_quad(PaintQuad::filled(selection_rect, self.selection_color));
            }
        }

        // Paint text
        let text_color = if is_placeholder {
            self.placeholder_color
        } else if self.disabled {
            colors::GRAY_500
        } else {
            self.text_style.color
        };

        // Center text vertically
        let text_size = ctx.text_system.measure_text(
            display_text,
            &text_config,
            None,
            ctx.scale_factor,
        );
        let text_y = text_area.pos.y + (text_area.size.y - text_size.y) / 2.0;

        ctx.paint_text(PaintText {
            position: Vec2::new(text_area.pos.x, text_y),
            text: display_text.to_string(),
            style: TextStyle {
                color: text_color,
                ..self.text_style.clone()
            },
        });

        // Paint cursor if focused and visible
        if is_focused && cursor_visible && !self.disabled && !is_placeholder {
            let text_before_cursor = &text[..cursor.min(text.len())];
            let cursor_x = if text_before_cursor.is_empty() {
                0.0
            } else {
                ctx.text_system.measure_text(text_before_cursor, &text_config, None, ctx.scale_factor).x
            };

            let cursor_rect = Rect::from_pos_size(
                Vec2::new(text_area.pos.x + cursor_x, text_area.pos.y + 2.0),
                Vec2::new(2.0, text_area.size.y - 4.0),
            );
            ctx.paint_quad(PaintQuad::filled(cursor_rect, self.cursor_color));
        } else if is_focused && !self.disabled && is_placeholder {
            // Show cursor at start when empty
            let cursor_rect = Rect::from_pos_size(
                Vec2::new(text_area.pos.x, text_area.pos.y + 2.0),
                Vec2::new(2.0, text_area.size.y - 4.0),
            );
            ctx.paint_quad(PaintQuad::filled(cursor_rect, self.cursor_color));
        }

        // Update cursor blink
        if is_focused {
            update_entity(&self.state, |s| {
                s.blink_counter += 1;
                if s.blink_counter >= 30 {
                    s.cursor_visible = !s.cursor_visible;
                    s.blink_counter = 0;
                }
            });
        } else {
            // Reset cursor visibility when not focused
            update_entity(&self.state, |s| {
                s.cursor_visible = true;
                s.blink_counter = 0;
            });
        }

        // Register for hit testing
        if !self.disabled {
            ctx.register_hit_test(self.element_id, bounds, 0);
        }
    }
}

/// An interactive text input that handles keyboard events
pub struct InteractiveTextInput {
    inner: InteractiveElement<TextInput>,
    #[allow(dead_code)]
    state: Entity<TextInputState>,
    #[allow(dead_code)]
    on_change: Option<Rc<RefCell<Box<dyn FnMut(&str)>>>>,
    #[allow(dead_code)]
    on_submit: Option<Rc<RefCell<Box<dyn FnMut(&str)>>>>,
}

impl InteractiveTextInput {
    pub fn new(input: TextInput) -> Self {
        let state = input.state.clone();
        let element_id = input.element_id;
        let disabled = input.disabled;
        let on_change = input.on_change.clone();
        let on_submit = input.on_submit.clone();
        let focus_border_color = input.focus_border_color;

        let state_for_keys = state.clone();
        let on_change_for_keys = on_change.clone();
        let on_submit_for_keys = on_submit.clone();

        let mut interactive = input
            .interactive()
            .with_id(element_id)
            .focusable_with_overlay(focus_border_color.with_alpha(0.1))
            .hover_overlay(colors::BLACK.with_alpha(0.02));

        if !disabled {
            interactive = interactive
                .on_key_down(move |key, modifiers, character, _is_repeat| {
                    let mut text_changed = false;

                    update_entity(&state_for_keys, |s| {
                        // Reset cursor blink on any key
                        s.cursor_visible = true;
                        s.blink_counter = 0;

                        match key {
                            Key::Backspace => {
                                s.backspace();
                                text_changed = true;
                            }
                            Key::Delete => {
                                s.delete();
                                text_changed = true;
                            }
                            Key::Left => {
                                s.move_left(modifiers.shift);
                            }
                            Key::Right => {
                                s.move_right(modifiers.shift);
                            }
                            Key::Home => {
                                s.move_to_start(modifiers.shift);
                            }
                            Key::End => {
                                s.move_to_end(modifiers.shift);
                            }
                            Key::A if modifiers.cmd => {
                                s.select_all();
                            }
                            Key::Return => {
                                // Don't modify text, just trigger submit
                            }
                            _ => {
                                // Handle character input
                                if let Some(c) = character {
                                    if !modifiers.cmd && !modifiers.ctrl {
                                        s.insert(&c.to_string());
                                        text_changed = true;
                                    }
                                }
                            }
                        }
                    });

                    // Call on_change if text was modified
                    if text_changed {
                        if let Some(handler) = &on_change_for_keys {
                            if let Some(text) = read_entity(&state_for_keys, |s| s.text.clone()) {
                                (handler.borrow_mut())(&text);
                            }
                        }
                    }

                    // Call on_submit for Enter key
                    if key == Key::Return {
                        if let Some(handler) = &on_submit_for_keys {
                            if let Some(text) = read_entity(&state_for_keys, |s| s.text.clone()) {
                                (handler.borrow_mut())(&text);
                            }
                        }
                    }
                })
                .on_focus_in({
                    let state = state.clone();
                    move || {
                        update_entity(&state, |s| {
                            s.cursor_visible = true;
                            s.blink_counter = 0;
                        });
                    }
                });
        } else {
            interactive = interactive.enabled(false);
        }

        Self {
            inner: interactive,
            state,
            on_change,
            on_submit,
        }
    }
}

impl Element for InteractiveTextInput {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        self.inner.layout(ctx)
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        self.inner.paint(bounds, ctx);
    }
}

/// Extension trait to make TextInput interactive
pub trait TextInputInteractable {
    /// Make this text input interactive with keyboard handling
    fn interactive_input(self) -> InteractiveTextInput;
}

impl TextInputInteractable for TextInput {
    fn interactive_input(self) -> InteractiveTextInput {
        InteractiveTextInput::new(self)
    }
}
