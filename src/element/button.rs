use crate::{
    color::{colors, Color},
    element::{Element, LayoutContext, PaintContext},
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
use taffy::prelude::*;

/// Create a new button element with the given label
pub fn button(label: impl Into<String>) -> Button {
    Button::new(label)
}

/// A clickable button element with text label
pub struct Button {
    /// Button text label
    label: String,

    /// Unique ID for interaction tracking
    id: ElementId,

    /// Event handlers
    handlers: Rc<RefCell<EventHandlers>>,

    /// Background color in normal state
    background: Color,

    /// Background color when hovered
    hover_background: Color,

    /// Background color when pressed
    press_background: Color,

    /// Background color when disabled
    disabled_background: Color,

    /// Border color
    border_color: Option<Color>,

    /// Border width
    border_width: f32,

    /// Corner radius
    corner_radius: f32,

    /// Text style
    text_style: TextStyle,

    /// Text color when disabled
    disabled_text_color: Color,

    /// Padding around the text
    padding_h: f32,
    padding_v: f32,

    /// Whether the button is disabled
    disabled: bool,

    /// Explicit width (None = auto-size to content)
    width: Option<taffy::Dimension>,

    /// Explicit height (None = auto-size to content)
    height: Option<taffy::Dimension>,

    /// Flex grow factor
    flex_grow: f32,

    /// Cached layout node ID
    node_id: Option<NodeId>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            id: ElementId::auto(),
            handlers: Rc::new(RefCell::new(EventHandlers::new())),
            background: colors::BLUE_500,
            hover_background: colors::BLUE_400,
            press_background: colors::BLUE_600,
            disabled_background: colors::GRAY_400,
            border_color: None,
            border_width: 0.0,
            corner_radius: 4.0,
            text_style: TextStyle {
                size: 14.0,
                color: colors::WHITE,
                ..Default::default()
            },
            disabled_text_color: colors::GRAY_600,
            padding_h: 16.0,
            padding_v: 8.0,
            disabled: false,
            width: None,
            height: None,
            flex_grow: 0.0,
            node_id: None,
        }
    }

    /// Set the element ID (useful for stable targeting)
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Set the background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set the hover background color
    pub fn hover_background(mut self, color: Color) -> Self {
        self.hover_background = color;
        self
    }

    /// Set the press background color
    pub fn press_background(mut self, color: Color) -> Self {
        self.press_background = color;
        self
    }

    /// Set all background colors at once (normal, hover, pressed)
    pub fn backgrounds(mut self, normal: Color, hover: Color, pressed: Color) -> Self {
        self.background = normal;
        self.hover_background = hover;
        self.press_background = pressed;
        self
    }

    /// Set the disabled background color
    pub fn disabled_background(mut self, color: Color) -> Self {
        self.disabled_background = color;
        self
    }

    /// Set the border
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.border_color = Some(color);
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

    /// Set disabled text color
    pub fn disabled_text_color(mut self, color: Color) -> Self {
        self.disabled_text_color = color;
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

    /// Set both horizontal and vertical padding
    pub fn padding(mut self, horizontal: f32, vertical: f32) -> Self {
        self.padding_h = horizontal;
        self.padding_v = vertical;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set explicit width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(taffy::Dimension::length(width));
        self
    }

    /// Set explicit height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(taffy::Dimension::length(height));
        self
    }

    /// Set both width and height
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = Some(taffy::Dimension::length(width));
        self.height = Some(taffy::Dimension::length(height));
        self
    }

    /// Set width to 100%
    pub fn width_full(mut self) -> Self {
        self.width = Some(taffy::Dimension::percent(1.0));
        self
    }

    /// Set height to 100%
    pub fn height_full(mut self) -> Self {
        self.height = Some(taffy::Dimension::percent(1.0));
        self
    }

    /// Set flex grow factor
    pub fn flex_grow(mut self, factor: f32) -> Self {
        self.flex_grow = factor;
        self
    }

    /// Set the click handler (also triggers on Enter/Space when focused)
    pub fn on_click<F>(self, handler: F) -> Self
    where
        F: FnMut(MouseButton, Vec2, Vec2) + 'static,
    {
        self.handlers.borrow_mut().on_click = Some(Box::new(handler));
        self
    }

    /// Set a simple click handler that doesn't need position info
    /// This also triggers on Enter/Space when the button is focused.
    pub fn on_click_simple<F>(self, handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        // Use Rc<RefCell> to share the handler between click and keyboard events
        let handler = Rc::new(RefCell::new(handler));
        let click_handler = handler.clone();
        let key_handler = handler;

        let mut handlers = self.handlers.borrow_mut();
        handlers.on_click = Some(Box::new(move |_, _, _| {
            (click_handler.borrow_mut())();
        }));

        // Also trigger on Enter or Space key
        handlers.on_key_down = Some(Box::new(move |key, _, _, is_repeat| {
            if !is_repeat && (key == Key::Return || key == Key::Space) {
                (key_handler.borrow_mut())();
            }
        }));

        drop(handlers);
        self
    }

    /// Set the mouse enter handler
    pub fn on_mouse_enter<F>(self, handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.handlers.borrow_mut().on_mouse_enter = Some(Box::new(handler));
        self
    }

    /// Set the mouse leave handler
    pub fn on_mouse_leave<F>(self, handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.handlers.borrow_mut().on_mouse_leave = Some(Box::new(handler));
        self
    }

    /// Get the element's ID
    pub fn element_id(&self) -> ElementId {
        self.id
    }
}

/// Focus ring color for buttons
const FOCUS_RING_COLOR: Color = colors::BLUE_400;
/// Focus ring width
const FOCUS_RING_WIDTH: f32 = 2.0;
/// Focus ring offset from element bounds
const FOCUS_RING_OFFSET: f32 = 2.0;

impl Element for Button {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Create style with padding and optional size constraints
        let style = Style {
            padding: taffy::Rect {
                left: LengthPercentage::length(self.padding_h),
                right: LengthPercentage::length(self.padding_h),
                top: LengthPercentage::length(self.padding_v),
                bottom: LengthPercentage::length(self.padding_v),
            },
            size: taffy::Size {
                width: self.width.unwrap_or(taffy::Dimension::auto()),
                height: self.height.unwrap_or(taffy::Dimension::auto()),
            },
            flex_grow: self.flex_grow,
            ..Default::default()
        };

        // Request text layout (button sizes to fit text + padding)
        let node_id = ctx.request_text_layout(style, &self.label, &self.text_style);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        // Register for interaction if not disabled
        if !self.disabled {
            register_element(self.id, self.handlers.clone());
        }

        // Get current interaction state
        let state = get_element_state(self.id).unwrap_or_default();

        // Paint focus ring if focused (paint before background so it appears behind)
        if state.is_focused && !self.disabled {
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

        // Determine background color based on state
        let bg_color = if self.disabled {
            self.disabled_background
        } else if state.is_pressed {
            self.press_background
        } else if state.is_hovered {
            self.hover_background
        } else {
            self.background
        };

        // Paint background
        ctx.paint_quad(PaintQuad {
            bounds,
            fill: bg_color,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::all(self.border_width),
            border_color: self.border_color.unwrap_or(colors::TRANSPARENT),
        });

        // Calculate text position (centered within bounds)
        let text_size = ctx.text_system.measure_text(
            &self.label,
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

        let text_x = bounds.pos.x + (bounds.size.x - text_size.x) / 2.0;
        let text_y = bounds.pos.y + (bounds.size.y - text_size.y) / 2.0;

        // Paint text
        let text_color = if self.disabled {
            self.disabled_text_color
        } else {
            self.text_style.color
        };

        ctx.paint_text(PaintText {
            position: Vec2::new(text_x, text_y),
            text: self.label.clone(),
            style: TextStyle {
                color: text_color,
                ..self.text_style.clone()
            },
            measured_size: Some(text_size),
        });

        // Register as focusable for hit testing if not disabled
        if !self.disabled {
            ctx.register_focusable(self.id, bounds, 0);
        }
    }
}
