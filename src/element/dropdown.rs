//! Dropdown/select element with customizable styling

use crate::{
    color::{colors, Color, ColorExt},
    element::{Element, LayoutContext, PaintContext},
    entity::{read_entity, update_entity, Entity},
    geometry::{Corners, Edges, Rect},
    interaction::{
        registry::{get_element_state, register_element},
        ElementId, EventHandlers,
    },
    render::{PaintQuad, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Create a new dropdown element with the given options
pub fn dropdown<T: Clone + ToString + 'static>(options: Vec<T>) -> Dropdown<T> {
    Dropdown::new(options)
}

/// State for dropdown element (persisted via Entity system)
#[derive(Clone, Default)]
pub struct DropdownState {
    /// Whether the dropdown is currently open
    pub is_open: bool,
    /// Index of highlighted option (for keyboard navigation)
    pub highlighted_index: Option<usize>,
}

/// A dropdown/select element
pub struct Dropdown<T: Clone + ToString + 'static> {
    /// Available options
    options: Vec<T>,
    /// Currently selected index
    selected_index: Option<usize>,
    /// Placeholder text when nothing selected
    placeholder: String,
    /// Whether the dropdown is disabled
    disabled: bool,

    // Styling
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
    /// Text style for selected value
    text_style: TextStyle,
    /// Text style for placeholder
    placeholder_style: TextStyle,
    /// Dropdown list background
    list_background: Color,
    /// Option hover background
    option_hover_background: Color,
    /// Option height
    option_height: f32,
    /// Padding
    padding_h: f32,
    padding_v: f32,
    /// Max visible options before scrolling
    max_visible_options: usize,

    // Internal
    /// Element ID for interaction tracking
    element_id: ElementId,
    /// Element IDs for each option
    option_ids: Vec<ElementId>,
    /// Event handlers
    handlers: Rc<RefCell<EventHandlers>>,
    /// On change callback
    on_change: Option<Rc<RefCell<Box<dyn FnMut(usize, &T)>>>>,
    /// Entity for persistent state
    state: Option<Entity<DropdownState>>,
    /// Cached layout node ID
    node_id: Option<NodeId>,
}

impl<T: Clone + ToString + 'static> Dropdown<T> {
    /// Create a new dropdown with the given options
    pub fn new(options: Vec<T>) -> Self {
        let option_count = options.len();
        Self {
            options,
            selected_index: None,
            placeholder: "Select...".to_string(),
            disabled: false,
            background: colors::WHITE,
            hover_background: colors::GRAY_100,
            open_background: colors::WHITE,
            border_color: colors::GRAY_300,
            border_width: 1.0,
            corner_radius: 4.0,
            text_style: TextStyle {
                size: 14.0,
                color: colors::BLACK,
                line_height: 1.2,
            },
            placeholder_style: TextStyle {
                size: 14.0,
                color: colors::GRAY_500,
                line_height: 1.2,
            },
            list_background: colors::WHITE,
            option_hover_background: Color::rgba(0.9, 0.95, 1.0, 1.0), // Light blue
            option_height: 32.0,
            padding_h: 12.0,
            padding_v: 8.0,
            max_visible_options: 6,
            element_id: ElementId::auto(),
            option_ids: (0..option_count).map(|_| ElementId::auto()).collect(),
            handlers: Rc::new(RefCell::new(EventHandlers::new())),
            on_change: None,
            state: None,
            node_id: None,
        }
    }

    /// Set the selected index
    pub fn selected(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    /// Set placeholder text
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

    /// Set on change handler
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: FnMut(usize, &T) + 'static,
    {
        self.on_change = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Bind to entity state for persistence
    pub fn state(mut self, state: Entity<DropdownState>) -> Self {
        self.state = Some(state);
        self
    }

    /// Get whether dropdown is open
    fn is_open(&self) -> bool {
        self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.is_open))
            .unwrap_or(false)
    }

    /// Set open state
    fn set_open(&self, open: bool) {
        if let Some(ref state) = self.state {
            update_entity(state, |s| {
                s.is_open = open;
                if !open {
                    s.highlighted_index = None;
                }
            });
        }
    }

    /// Get highlighted index
    fn highlighted_index(&self) -> Option<usize> {
        self.state
            .as_ref()
            .and_then(|s| read_entity(s, |state| state.highlighted_index))
            .unwrap_or(None)
    }

    /// Set highlighted index
    fn set_highlighted(&self, index: Option<usize>) {
        if let Some(ref state) = self.state {
            update_entity(state, |s| {
                s.highlighted_index = index;
            });
        }
    }
}

impl<T: Clone + ToString + 'static> Element for Dropdown<T> {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Dropdown trigger button layout - fixed height with padding
        let min_height = self.padding_v * 2.0 + self.text_style.size * 1.2;
        let style = Style {
            size: Size {
                width: Dimension::length(120.0),
                height: Dimension::length(min_height),
            },
            padding: taffy::Rect {
                left: LengthPercentage::length(self.padding_h),
                right: LengthPercentage::length(self.padding_h + 20.0), // Extra for arrow
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

        let is_open = self.is_open();
        let state = get_element_state(self.element_id).unwrap_or_default();

        // Determine background color
        let bg_color = if self.disabled {
            colors::GRAY_200
        } else if is_open {
            self.open_background
        } else if state.is_hovered {
            self.hover_background
        } else {
            self.background
        };

        // Paint trigger button background
        ctx.paint_quad(PaintQuad {
            bounds,
            fill: bg_color,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::all(self.border_width),
            border_color: if is_open { colors::BLUE_500 } else { self.border_color },
        });

        // Paint selected text or placeholder
        let display_text = if let Some(idx) = self.selected_index {
            self.options.get(idx).map(|o| o.to_string())
        } else {
            None
        };

        let (text, style) = match display_text {
            Some(t) => (t, &self.text_style),
            None => (self.placeholder.clone(), &self.placeholder_style),
        };

        let text_y = bounds.pos.y + (bounds.size.y - style.size * 1.2) / 2.0;
        ctx.paint_text(PaintText {
            position: Vec2::new(bounds.pos.x + self.padding_h, text_y),
            text,
            style: style.clone(),
        });

        // Paint dropdown arrow
        let arrow_x = bounds.pos.x + bounds.size.x - self.padding_h - 8.0;
        let arrow_y = bounds.pos.y + bounds.size.y / 2.0;
        let arrow_text = if is_open { "\u{25B2}" } else { "\u{25BC}" }; // Up/down triangles
        ctx.paint_text(PaintText {
            position: Vec2::new(arrow_x, arrow_y - 6.0),
            text: arrow_text.to_string(),
            style: TextStyle {
                size: 10.0,
                color: colors::GRAY_600,
                line_height: 1.2,
            },
        });

        // Register click handler for trigger
        if !self.disabled {
            // Create click handler that toggles open state
            let state_ref = self.state.clone();
            let mut handlers = EventHandlers::new();
            handlers.on_click = Some(Box::new(move |_, _, _| {
                if let Some(ref state) = state_ref {
                    update_entity(state, |s| {
                        s.is_open = !s.is_open;
                    });
                }
            }));
            register_element(self.element_id, Rc::new(RefCell::new(handlers)));
            ctx.register_hit_test(self.element_id, bounds, 0);
        }

        // Paint dropdown list if open
        if is_open && !self.options.is_empty() {
            let visible_count = self.options.len().min(self.max_visible_options);
            let list_height = visible_count as f32 * self.option_height;

            let list_bounds = Rect::from_pos_size(
                Vec2::new(bounds.pos.x, bounds.pos.y + bounds.size.y + 2.0),
                Vec2::new(bounds.size.x, list_height),
            );

            // Paint list background with shadow effect
            ctx.paint_quad(PaintQuad {
                bounds: list_bounds,
                fill: self.list_background,
                corner_radii: Corners::all(self.corner_radius),
                border_widths: Edges::all(1.0),
                border_color: colors::GRAY_300,
            });

            // Paint options
            let highlighted = self.highlighted_index();
            for (i, option) in self.options.iter().enumerate().take(visible_count) {
                let option_bounds = Rect::from_pos_size(
                    Vec2::new(
                        list_bounds.pos.x,
                        list_bounds.pos.y + i as f32 * self.option_height,
                    ),
                    Vec2::new(list_bounds.size.x, self.option_height),
                );

                let option_state = get_element_state(self.option_ids[i]).unwrap_or_default();
                let is_selected = self.selected_index == Some(i);
                let is_highlighted = highlighted == Some(i);

                // Option background
                let option_bg = if is_selected {
                    Color::rgba(0.9, 0.95, 1.0, 1.0) // Light blue for selected
                } else if is_highlighted || option_state.is_hovered {
                    self.option_hover_background
                } else {
                    colors::TRANSPARENT
                };

                if option_bg.alpha > 0.0 {
                    ctx.paint_quad(PaintQuad {
                        bounds: option_bounds,
                        fill: option_bg,
                        corner_radii: Corners::zero(),
                        border_widths: Edges::zero(),
                        border_color: colors::TRANSPARENT,
                    });
                }

                // Option text
                let option_text_y = option_bounds.pos.y + (self.option_height - self.text_style.size * 1.2) / 2.0;
                ctx.paint_text(PaintText {
                    position: Vec2::new(option_bounds.pos.x + self.padding_h, option_text_y),
                    text: option.to_string(),
                    style: TextStyle {
                        color: if is_selected { colors::BLUE_600 } else { colors::BLACK },
                        ..self.text_style.clone()
                    },
                });

                // Register option click handler
                let selected_index = i;
                let state_ref = self.state.clone();
                let on_change = self.on_change.clone();
                let option_value = option.clone();

                let mut option_handlers = EventHandlers::new();
                option_handlers.on_click = Some(Box::new(move |_, _, _| {
                    // Close dropdown
                    if let Some(ref state) = state_ref {
                        update_entity(state, |s| {
                            s.is_open = false;
                            s.highlighted_index = None;
                        });
                    }
                    // Call on_change handler
                    if let Some(ref handler) = on_change {
                        handler.borrow_mut()(selected_index, &option_value);
                    }
                }));

                register_element(self.option_ids[i], Rc::new(RefCell::new(option_handlers)));
                ctx.register_hit_test(self.option_ids[i], option_bounds, 1); // Higher z for options
            }
        }
    }
}
