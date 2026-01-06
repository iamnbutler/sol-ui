//! Dropdown/select element with keyboard navigation and type-ahead search

use crate::{
    color::{colors, Color, ColorExt},
    element::{Element, LayoutContext, PaintContext},
    entity::{new_entity, read_entity, update_entity, Entity},
    geometry::{Corners, Edges, Rect},
    interaction::{
        registry::{get_element_state, register_element},
        ElementId, EventHandlers,
    },
    layer::{Key, MouseButton},
    render::{PaintQuad, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;
use taffy::prelude::*;

/// State for a dropdown, persisted via the Entity system
#[derive(Debug, Clone)]
pub struct DropdownState {
    /// Whether the dropdown is currently open
    pub is_open: bool,
    /// Index of currently selected option (None if nothing selected)
    pub selected_index: Option<usize>,
    /// Index of currently highlighted option (for keyboard navigation)
    pub highlighted_index: Option<usize>,
    /// Current search/filter text for type-ahead
    pub search_text: String,
    /// Timestamp of last search keystroke (for clearing search)
    pub last_search_time: Option<Instant>,
}

impl Default for DropdownState {
    fn default() -> Self {
        Self {
            is_open: false,
            selected_index: None,
            highlighted_index: None,
            search_text: String::new(),
            last_search_time: None,
        }
    }
}

impl DropdownState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle the dropdown open/closed
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
        if self.is_open {
            // Reset highlighted to selected when opening
            self.highlighted_index = self.selected_index;
        } else {
            // Clear search when closing
            self.search_text.clear();
        }
    }

    /// Close the dropdown
    pub fn close(&mut self) {
        self.is_open = false;
        self.search_text.clear();
    }

    /// Select an option by index
    pub fn select(&mut self, index: usize) {
        self.selected_index = Some(index);
        self.close();
    }
}

/// A single option in the dropdown
#[derive(Debug, Clone)]
pub struct DropdownOption<T> {
    /// The value associated with this option
    pub value: T,
    /// Display label (if different from value.to_string())
    pub label: Option<String>,
    /// Whether this option is disabled
    pub disabled: bool,
    /// Optional group this option belongs to
    pub group: Option<String>,
}

impl<T: ToString + Clone> DropdownOption<T> {
    /// Create a new option from a value
    pub fn new(value: T) -> Self {
        Self {
            value,
            label: None,
            disabled: false,
            group: None,
        }
    }

    /// Set a custom label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Mark this option as disabled
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }

    /// Set the group for this option
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Get the display text for this option
    pub fn display_text(&self) -> String {
        self.label
            .clone()
            .unwrap_or_else(|| self.value.to_string())
    }
}

/// Create a new dropdown with the given options
pub fn dropdown<T: ToString + Clone + 'static>(options: Vec<T>) -> Dropdown<T> {
    Dropdown::new(options)
}

/// A dropdown/select element
pub struct Dropdown<T: ToString + Clone + 'static> {
    /// The options available for selection
    options: Vec<DropdownOption<T>>,
    /// Placeholder text when nothing is selected
    placeholder: String,
    /// Element ID for interaction tracking
    element_id: ElementId,
    /// Element ID for the options list (separate hit area)
    options_element_id: ElementId,
    /// Event handlers for the trigger
    handlers: Rc<RefCell<EventHandlers>>,
    /// Persistent state entity
    state: Option<Entity<DropdownState>>,
    /// On change callback
    on_change: Option<Rc<RefCell<Box<dyn FnMut(usize, &T)>>>>,

    // Styling
    /// Width of the dropdown
    width: f32,
    /// Max height of the options list
    max_options_height: f32,
    /// Background color
    background: Color,
    /// Background color when hovered
    hover_background: Color,
    /// Background color when open
    open_background: Color,
    /// Border color
    border_color: Color,
    /// Border width
    border_width: f32,
    /// Corner radius
    corner_radius: f32,
    /// Text style for the selected value
    text_style: TextStyle,
    /// Text style for placeholder
    placeholder_style: TextStyle,
    /// Text style for options
    option_style: TextStyle,
    /// Background for highlighted option
    highlight_background: Color,
    /// Background for selected option in list
    selected_background: Color,
    /// Disabled option text color
    disabled_color: Color,
    /// Padding
    padding_h: f32,
    padding_v: f32,
    /// Option padding
    option_padding_h: f32,
    option_padding_v: f32,

    /// Whether the dropdown is disabled
    disabled: bool,

    /// Cached layout node
    node_id: Option<NodeId>,
}

impl<T: ToString + Clone + 'static> Dropdown<T> {
    /// Create a new dropdown with simple value options
    pub fn new(options: Vec<T>) -> Self {
        let options: Vec<DropdownOption<T>> =
            options.into_iter().map(DropdownOption::new).collect();

        Self {
            options,
            placeholder: "Select...".into(),
            element_id: ElementId::auto(),
            options_element_id: ElementId::auto(),
            handlers: Rc::new(RefCell::new(EventHandlers::new())),
            state: None,
            on_change: None,
            width: 200.0,
            max_options_height: 300.0,
            background: colors::WHITE,
            hover_background: colors::GRAY_100,
            open_background: colors::WHITE,
            border_color: colors::GRAY_300,
            border_width: 1.0,
            corner_radius: 4.0,
            text_style: TextStyle {
                size: 14.0,
                color: colors::BLACK,
            },
            placeholder_style: TextStyle {
                size: 14.0,
                color: colors::GRAY_500,
            },
            option_style: TextStyle {
                size: 14.0,
                color: colors::BLACK,
            },
            highlight_background: colors::BLUE_400.with_alpha(0.2),
            selected_background: colors::BLUE_400.with_alpha(0.1),
            disabled_color: colors::GRAY_400,
            padding_h: 12.0,
            padding_v: 8.0,
            option_padding_h: 12.0,
            option_padding_v: 8.0,
            disabled: false,
            node_id: None,
        }
    }

    /// Create a new dropdown with rich options
    pub fn with_options(options: Vec<DropdownOption<T>>) -> Self {
        Self {
            options,
            ..Self::new(vec![])
        }
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set a stable element ID
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Bind to a persistent state entity
    pub fn state(mut self, state: Entity<DropdownState>) -> Self {
        self.state = Some(state);
        self
    }

    /// Set the initial selected index
    pub fn selected(mut self, index: usize) -> Self {
        if self.state.is_none() {
            self.state = Some(new_entity(DropdownState::new()));
        }
        if let Some(ref state) = self.state {
            update_entity(state, |s| s.selected_index = Some(index));
        }
        self
    }

    /// Set the on_change callback
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(usize, &T) + 'static,
    {
        self.on_change = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Set the width
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Set the max height for the options list
    pub fn max_options_height(mut self, height: f32) -> Self {
        self.max_options_height = height;
        self
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set hover background color
    pub fn hover_background(mut self, color: Color) -> Self {
        self.hover_background = color;
        self
    }

    /// Set border
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = color;
        self.border_width = width;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set text style
    pub fn text_style(mut self, style: TextStyle) -> Self {
        self.text_style = style;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Get the current state
    fn get_state(&self) -> DropdownState {
        self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.clone()))
            .unwrap_or_default()
    }

    /// Get the currently selected option
    pub fn selected_option(&self) -> Option<&DropdownOption<T>> {
        let state = self.get_state();
        state.selected_index.and_then(|i| self.options.get(i))
    }

    /// Filter options based on search text (used by type-ahead search)
    #[allow(dead_code)]
    fn filtered_options(&self, search: &str) -> Vec<(usize, &DropdownOption<T>)> {
        if search.is_empty() {
            self.options.iter().enumerate().collect()
        } else {
            let search_lower = search.to_lowercase();
            self.options
                .iter()
                .enumerate()
                .filter(|(_, opt)| {
                    opt.display_text().to_lowercase().contains(&search_lower)
                })
                .collect()
        }
    }

    /// Handle keyboard navigation (called when dropdown has focus)
    #[allow(dead_code)]
    fn handle_key(&self, key: Key, state: &mut DropdownState) {
        if !state.is_open {
            // Space or Enter opens the dropdown
            if matches!(key, Key::Space | Key::Return) {
                state.toggle();
            }
            return;
        }

        match key {
            Key::Escape => {
                state.close();
            }
            Key::Return => {
                // Select highlighted option
                if let Some(idx) = state.highlighted_index {
                    if !self.options.get(idx).map(|o| o.disabled).unwrap_or(true) {
                        state.select(idx);
                        if let Some(ref on_change) = self.on_change {
                            if let Some(opt) = self.options.get(idx) {
                                (on_change.borrow_mut())(idx, &opt.value);
                            }
                        }
                    }
                }
            }
            Key::Up => {
                // Move highlight up
                let current = state.highlighted_index.unwrap_or(0);
                if current > 0 {
                    // Find previous non-disabled option
                    for i in (0..current).rev() {
                        if !self.options.get(i).map(|o| o.disabled).unwrap_or(true) {
                            state.highlighted_index = Some(i);
                            break;
                        }
                    }
                }
            }
            Key::Down => {
                // Move highlight down
                let current = state.highlighted_index.unwrap_or(0);
                let max = self.options.len().saturating_sub(1);
                if current < max {
                    // Find next non-disabled option
                    for i in (current + 1)..=max {
                        if !self.options.get(i).map(|o| o.disabled).unwrap_or(true) {
                            state.highlighted_index = Some(i);
                            break;
                        }
                    }
                }
            }
            Key::Home => {
                // Jump to first non-disabled option
                for (i, opt) in self.options.iter().enumerate() {
                    if !opt.disabled {
                        state.highlighted_index = Some(i);
                        break;
                    }
                }
            }
            Key::End => {
                // Jump to last non-disabled option
                for (i, opt) in self.options.iter().enumerate().rev() {
                    if !opt.disabled {
                        state.highlighted_index = Some(i);
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle character input for type-ahead search (called when dropdown has focus)
    #[allow(dead_code)]
    fn handle_char(&self, ch: char, state: &mut DropdownState) {
        if !state.is_open {
            return;
        }

        // Clear search if too much time has passed (500ms)
        let now = Instant::now();
        if let Some(last_time) = state.last_search_time {
            if now.duration_since(last_time).as_millis() > 500 {
                state.search_text.clear();
            }
        }

        // Add character to search
        state.search_text.push(ch);
        state.last_search_time = Some(now);

        // Find first matching option
        let search_lower = state.search_text.to_lowercase();
        for (i, opt) in self.options.iter().enumerate() {
            if !opt.disabled && opt.display_text().to_lowercase().starts_with(&search_lower) {
                state.highlighted_index = Some(i);
                break;
            }
        }
    }

    /// Paint the dropdown trigger (closed state display)
    fn paint_trigger(&self, bounds: Rect, ctx: &mut PaintContext, state: &DropdownState) {
        let interaction_state = get_element_state(self.element_id).unwrap_or_default();

        // Determine background color
        let bg = if self.disabled {
            colors::GRAY_100
        } else if state.is_open {
            self.open_background
        } else if interaction_state.is_hovered {
            self.hover_background
        } else {
            self.background
        };

        // Paint background
        ctx.paint_quad(PaintQuad {
            bounds,
            fill: bg,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::all(self.border_width),
            border_color: if state.is_open {
                colors::BLUE_500
            } else {
                self.border_color
            },
        });

        // Paint selected text or placeholder
        let (text, style) = if let Some(idx) = state.selected_index {
            if let Some(opt) = self.options.get(idx) {
                (opt.display_text(), self.text_style.clone())
            } else {
                (self.placeholder.clone(), self.placeholder_style.clone())
            }
        } else {
            (self.placeholder.clone(), self.placeholder_style.clone())
        };

        let text_x = bounds.pos.x + self.padding_h;
        let text_y = bounds.pos.y + (bounds.size.y - style.size) / 2.0;

        ctx.paint_text(PaintText {
            position: Vec2::new(text_x, text_y),
            text,
            style,
        });

        // Paint dropdown arrow
        let arrow_size = 8.0;
        let arrow_x = bounds.pos.x + bounds.size.x - self.padding_h - arrow_size;
        let arrow_y = bounds.pos.y + (bounds.size.y - arrow_size) / 2.0;

        // Simple triangle arrow using rectangles
        let arrow_color = if self.disabled {
            self.disabled_color
        } else {
            colors::GRAY_600
        };

        // Draw a simple down arrow (two small rects forming a V)
        let half = arrow_size / 2.0;
        ctx.paint_quad(PaintQuad::filled(
            Rect::from_pos_size(Vec2::new(arrow_x, arrow_y + 2.0), Vec2::new(half, 2.0)),
            arrow_color,
        ));
        ctx.paint_quad(PaintQuad::filled(
            Rect::from_pos_size(
                Vec2::new(arrow_x + half, arrow_y + 2.0),
                Vec2::new(half, 2.0),
            ),
            arrow_color,
        ));
        ctx.paint_quad(PaintQuad::filled(
            Rect::from_pos_size(
                Vec2::new(arrow_x + half / 2.0, arrow_y + 4.0),
                Vec2::new(half, 2.0),
            ),
            arrow_color,
        ));
    }

    /// Paint the options list (open state)
    fn paint_options(&mut self, trigger_bounds: Rect, ctx: &mut PaintContext, state: &DropdownState) {
        let option_height = self.option_style.size + self.option_padding_v * 2.0;
        let total_height = (self.options.len() as f32 * option_height).min(self.max_options_height);

        // Options list bounds (below trigger)
        let list_bounds = Rect::from_pos_size(
            Vec2::new(trigger_bounds.pos.x, trigger_bounds.pos.y + trigger_bounds.size.y + 2.0),
            Vec2::new(trigger_bounds.size.x, total_height),
        );

        // Paint options background with shadow effect
        ctx.paint_quad(PaintQuad {
            bounds: list_bounds,
            fill: colors::WHITE,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::all(1.0),
            border_color: colors::GRAY_200,
        });

        // Paint each option
        let mut y = list_bounds.pos.y;
        for (i, opt) in self.options.iter().enumerate() {
            if y + option_height > list_bounds.pos.y + list_bounds.size.y {
                break; // Exceeds max height
            }

            let option_bounds = Rect::from_pos_size(
                Vec2::new(list_bounds.pos.x, y),
                Vec2::new(list_bounds.size.x, option_height),
            );

            // Determine option background
            let is_highlighted = state.highlighted_index == Some(i);
            let is_selected = state.selected_index == Some(i);

            let bg = if opt.disabled {
                colors::TRANSPARENT
            } else if is_highlighted {
                self.highlight_background
            } else if is_selected {
                self.selected_background
            } else {
                colors::TRANSPARENT
            };

            if bg != colors::TRANSPARENT {
                ctx.paint_quad(PaintQuad::filled(option_bounds, bg));
            }

            // Paint option text
            let text_color = if opt.disabled {
                self.disabled_color
            } else {
                self.option_style.color
            };

            ctx.paint_text(PaintText {
                position: Vec2::new(
                    option_bounds.pos.x + self.option_padding_h,
                    option_bounds.pos.y + self.option_padding_v,
                ),
                text: opt.display_text(),
                style: TextStyle {
                    color: text_color,
                    ..self.option_style.clone()
                },
            });

            // Register hit area for this option (if not disabled)
            if !opt.disabled {
                // Create a unique ID for this option
                let option_id = ElementId::new(self.options_element_id.0 + i as u64 + 1);
                ctx.register_hit_test(option_id, option_bounds, 100); // High z-index for options
            }

            y += option_height;
        }

        // Register hit area for entire options list to capture clicks
        ctx.register_hit_test(self.options_element_id, list_bounds, 99);
    }
}

impl<T: ToString + Clone + 'static> Element for Dropdown<T> {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Initialize state if needed
        if self.state.is_none() {
            self.state = Some(new_entity(DropdownState::new()));
        }

        let trigger_height = self.text_style.size + self.padding_v * 2.0;

        let style = Style {
            size: Size {
                width: Dimension::length(self.width),
                height: Dimension::length(trigger_height),
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

        let state = self.get_state();

        // Register click handler for trigger
        if !self.disabled {
            let state_entity = self.state.clone();
            let _on_change = self.on_change.clone();
            let _options_len = self.options.len();

            self.handlers.borrow_mut().on_click = Some(Box::new(move |button, _, _| {
                if button == MouseButton::Left {
                    if let Some(ref entity) = state_entity {
                        update_entity(entity, |s| s.toggle());
                    }
                }
            }));

            register_element(self.element_id, self.handlers.clone());
        }

        // Paint trigger
        self.paint_trigger(bounds, ctx, &state);

        // Register trigger hit area
        if !self.disabled {
            ctx.register_hit_test(self.element_id, bounds, 0);
        }

        // Paint options list if open
        if state.is_open {
            self.paint_options(bounds, ctx, &state);
        }
    }
}
