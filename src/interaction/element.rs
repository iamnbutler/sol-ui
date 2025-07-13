//! Interactive element wrapper that adds interaction capabilities to any element

use super::ElementId;
use crate::color::Color;
use crate::element::{Element, LayoutContext, PaintContext};
use crate::geometry::Rect;
use crate::interaction::events::EventHandlers;
use crate::interaction::registry::{get_element_state, register_element};
use crate::paint::PaintQuad;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Wrapper that makes any element interactive
pub struct InteractiveElement<E: Element> {
    /// The wrapped element
    element: E,

    /// Unique ID for this interactive element
    id: ElementId,

    /// Event handlers
    handlers: Rc<RefCell<EventHandlers>>,

    /// Visual feedback options
    hover_overlay: Option<Color>,
    press_overlay: Option<Color>,

    /// Whether this element is interactive
    enabled: bool,

    /// Z-index offset for this element
    z_index: i32,

    /// Cached layout node ID
    node_id: Option<NodeId>,
}

impl<E: Element> InteractiveElement<E> {
    /// Create a new interactive element wrapper
    pub fn new(element: E) -> Self {
        Self {
            element,
            id: ElementId::auto(),
            handlers: Rc::new(RefCell::new(EventHandlers::new())),
            hover_overlay: None,
            press_overlay: None,
            enabled: true,
            z_index: 0,
            node_id: None,
        }
    }

    /// Set the element ID (useful for debugging or specific targeting)
    pub fn with_id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Set hover overlay color for visual feedback
    pub fn hover_overlay(mut self, color: Color) -> Self {
        self.hover_overlay = Some(color);
        self
    }

    /// Set press overlay color for visual feedback
    pub fn press_overlay(mut self, color: Color) -> Self {
        self.press_overlay = Some(color);
        self
    }

    /// Set both hover and press overlays
    pub fn with_overlays(mut self, hover: Color, press: Color) -> Self {
        self.hover_overlay = Some(hover);
        self.press_overlay = Some(press);
        self
    }

    /// Set whether this element is interactive
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the z-index offset for this element
    pub fn z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// Set the click handler
    pub fn on_click<F>(self, handler: F) -> Self
    where
        F: FnMut(crate::layer::MouseButton, glam::Vec2, glam::Vec2) + 'static,
    {
        self.handlers.borrow_mut().on_click = Some(Box::new(handler));
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

    /// Set the mouse move handler
    pub fn on_mouse_move<F>(self, handler: F) -> Self
    where
        F: FnMut(glam::Vec2, glam::Vec2) + 'static,
    {
        self.handlers.borrow_mut().on_mouse_move = Some(Box::new(handler));
        self
    }

    /// Set the mouse down handler
    pub fn on_mouse_down<F>(self, handler: F) -> Self
    where
        F: FnMut(crate::layer::MouseButton, glam::Vec2, glam::Vec2) + 'static,
    {
        self.handlers.borrow_mut().on_mouse_down = Some(Box::new(handler));
        self
    }

    /// Set the mouse up handler
    pub fn on_mouse_up<F>(self, handler: F) -> Self
    where
        F: FnMut(crate::layer::MouseButton, glam::Vec2, glam::Vec2) + 'static,
    {
        self.handlers.borrow_mut().on_mouse_up = Some(Box::new(handler));
        self
    }

    /// Get the element's ID
    pub fn element_id(&self) -> ElementId {
        self.id
    }
}

impl<E: Element> Element for InteractiveElement<E> {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Delegate layout to wrapped element
        let node_id = self.element.layout(ctx);
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        // First, paint the wrapped element
        self.element.paint(bounds, ctx);

        // Register this element with the current registry
        if self.enabled {
            register_element(self.id, self.handlers.clone());
        }

        // Get current interaction state from registry
        let state = get_element_state(self.id).unwrap_or_default();

        // Then apply interaction overlays if needed
        if self.enabled {
            let overlay_color = if state.is_pressed {
                self.press_overlay
            } else if state.is_hovered {
                self.hover_overlay
            } else {
                None
            };

            if let Some(color) = overlay_color {
                // Paint overlay on top of the element
                ctx.paint_quad(PaintQuad::filled(bounds, color));
            }
        }

        // Register this element for hit testing
        if self.enabled {
            ctx.register_hit_test(self.id, bounds, self.z_index);
        }
    }
}

/// Helper trait to make any element interactive
pub trait Interactable: Element + Sized {
    /// Wrap this element in an interactive wrapper
    fn interactive(self) -> InteractiveElement<Self> {
        InteractiveElement::new(self)
    }
}

// Implement Interactable for all Element types
impl<T: Element> Interactable for T {}

/// Create an interactive wrapper around any element with an auto-generated ID
pub fn interactive<E: Element>(element: E) -> InteractiveElement<E> {
    InteractiveElement::new(element)
}

/// Create an interactive wrapper with a specific stable ID
pub fn interactive_with_id<E: Element>(
    element: E,
    id: impl Into<ElementId>,
) -> InteractiveElement<E> {
    InteractiveElement::new(element).with_id(id.into())
}
