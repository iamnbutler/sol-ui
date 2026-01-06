//! Modal dialog element

use crate::{
    color::{colors, Color, ColorExt},
    element::{Element, LayoutContext},
    geometry::{Corners, Edges, Rect},
    interaction::{registry::register_element, ElementId, EventHandlers},
    layer::Key,
    render::{PaintContext, PaintQuad},
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Create a new modal element
pub fn modal() -> Modal {
    Modal::new()
}

/// A modal dialog with backdrop overlay
pub struct Modal {
    /// Whether the modal is visible
    is_open: bool,
    /// Backdrop color (semi-transparent overlay)
    backdrop_color: Color,
    /// Dialog background color
    dialog_background: Color,
    /// Dialog corner radius
    corner_radius: f32,
    /// Dialog padding
    padding: f32,
    /// Close on backdrop click
    close_on_backdrop: bool,
    /// Close on Escape key
    close_on_escape: bool,
    /// Close callback
    on_close: Option<Rc<RefCell<Box<dyn FnMut()>>>>,
    /// Child content
    child: Option<Box<dyn Element>>,
    /// Child node ID
    child_node: Option<NodeId>,
    /// Backdrop element ID for hit testing
    backdrop_id: ElementId,
    /// Dialog element ID
    dialog_id: ElementId,
    /// Event handlers for backdrop
    backdrop_handlers: Rc<RefCell<EventHandlers>>,
    /// Event handlers for dialog (captures escape)
    dialog_handlers: Rc<RefCell<EventHandlers>>,
}

impl Modal {
    pub fn new() -> Self {
        Self {
            is_open: false,
            backdrop_color: Color::rgba(0.0, 0.0, 0.0, 0.5),
            dialog_background: colors::WHITE,
            corner_radius: 8.0,
            padding: 24.0,
            close_on_backdrop: true,
            close_on_escape: true,
            on_close: None,
            child: None,
            child_node: None,
            backdrop_id: ElementId::auto(),
            dialog_id: ElementId::auto(),
            backdrop_handlers: Rc::new(RefCell::new(EventHandlers::new())),
            dialog_handlers: Rc::new(RefCell::new(EventHandlers::new())),
        }
    }

    /// Set whether the modal is open
    pub fn open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    /// Set the backdrop color
    pub fn backdrop_color(mut self, color: Color) -> Self {
        self.backdrop_color = color;
        self
    }

    /// Set the dialog background color
    pub fn background(mut self, color: Color) -> Self {
        self.dialog_background = color;
        self
    }

    /// Set corner radius
    pub fn corner_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    /// Set whether clicking the backdrop closes the modal
    pub fn close_on_backdrop(mut self, close: bool) -> Self {
        self.close_on_backdrop = close;
        self
    }

    /// Set whether Escape key closes the modal
    pub fn close_on_escape(mut self, close: bool) -> Self {
        self.close_on_escape = close;
        self
    }

    /// Set the close callback
    pub fn on_close<F>(mut self, handler: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.on_close = Some(Rc::new(RefCell::new(Box::new(handler))));
        self
    }

    /// Set the dialog content
    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

}

impl Default for Modal {
    fn default() -> Self {
        Self::new()
    }
}

impl Element for Modal {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Layout child if present
        if let Some(ref mut child) = self.child {
            self.child_node = Some(child.layout(ctx));
        }

        // Modal takes no space in the layout tree - it's an overlay
        ctx.request_layout(Style::default())
    }

    fn paint(&mut self, _bounds: Rect, ctx: &mut PaintContext) {
        if !self.is_open {
            return;
        }

        // Get viewport size for full-screen backdrop
        let viewport = ctx.draw_list.viewport().unwrap_or(Rect::from_pos_size(
            Vec2::ZERO,
            Vec2::new(800.0, 600.0), // Fallback size
        ));

        // Setup backdrop click handler
        if self.close_on_backdrop {
            let on_close = self.on_close.clone();
            self.backdrop_handlers.borrow_mut().on_click = Some(Box::new(move |_, _, _, _, _| {
                if let Some(ref handler) = on_close {
                    (handler.borrow_mut())();
                }
            }));
        }

        // Setup escape key handler on dialog
        if self.close_on_escape {
            let on_close = self.on_close.clone();
            self.dialog_handlers.borrow_mut().on_key_down = Some(Box::new(move |key, _, _, _| {
                if key == Key::Escape {
                    if let Some(ref handler) = on_close {
                        (handler.borrow_mut())();
                    }
                }
            }));
        }

        // Register backdrop for interaction (high z-index to capture all clicks)
        register_element(self.backdrop_id, self.backdrop_handlers.clone());

        // Paint backdrop (full viewport)
        ctx.paint_quad(PaintQuad {
            bounds: viewport,
            fill: self.backdrop_color,
            corner_radii: Corners::all(0.0),
            border_widths: Edges::zero(),
            border_color: colors::TRANSPARENT,
        });

        // Register backdrop for hit testing at high z-index
        ctx.register_hit_test(self.backdrop_id, viewport, 1000);

        // Calculate dialog size from child
        let dialog_size = if let Some(child_node) = self.child_node {
            let child_bounds = ctx.layout_engine.layout_bounds(child_node);
            Vec2::new(
                child_bounds.size.x + self.padding * 2.0,
                child_bounds.size.y + self.padding * 2.0,
            )
        } else {
            Vec2::new(300.0, 200.0) // Default size
        };

        // Center dialog in viewport
        let dialog_pos = Vec2::new(
            viewport.pos.x + (viewport.size.x - dialog_size.x) / 2.0,
            viewport.pos.y + (viewport.size.y - dialog_size.y) / 2.0,
        );

        let dialog_bounds = Rect::from_pos_size(dialog_pos, dialog_size);

        // Register dialog for interaction (higher z-index than backdrop)
        register_element(self.dialog_id, self.dialog_handlers.clone());

        // Paint dialog background
        ctx.paint_quad(PaintQuad {
            bounds: dialog_bounds,
            fill: self.dialog_background,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::zero(),
            border_color: colors::TRANSPARENT,
        });

        // Register dialog for hit testing (higher than backdrop to block clicks through)
        ctx.register_focusable(self.dialog_id, dialog_bounds, 1001);

        // Paint child content inside dialog
        if let Some(ref mut child) = self.child {
            let content_bounds = Rect::from_pos_size(
                dialog_pos + Vec2::splat(self.padding),
                dialog_size - Vec2::splat(self.padding * 2.0),
            );
            child.paint(content_bounds, ctx);
        }
    }
}
