//! Tooltip element - shows hint text on hover

use crate::{
    color::{colors, Color},
    element::{Element, LayoutContext},
    geometry::{Corners, Edges, Rect},
    interaction::{
        registry::{get_element_state, register_element},
        ElementId, EventHandlers,
    },
    render::{PaintContext, PaintQuad, PaintText},
    style::TextStyle,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Position for tooltip relative to target
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TooltipPosition {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

/// Create a tooltip wrapper
pub fn tooltip(text: impl Into<String>) -> Tooltip {
    Tooltip::new(text)
}

/// A tooltip wrapper that shows hint text on hover
pub struct Tooltip {
    /// Tooltip text
    text: String,
    /// Position relative to child
    position: TooltipPosition,
    /// Background color
    background: Color,
    /// Text color
    text_color: Color,
    /// Corner radius
    corner_radius: f32,
    /// Padding
    padding: f32,
    /// Gap between tooltip and target
    gap: f32,
    /// Child element
    child: Option<Box<dyn Element>>,
    /// Child node ID
    child_node: Option<NodeId>,
    /// Element ID for hover detection
    element_id: ElementId,
    /// Event handlers
    handlers: Rc<RefCell<EventHandlers>>,
}

impl Tooltip {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            position: TooltipPosition::Top,
            background: colors::GRAY_800,
            text_color: colors::WHITE,
            corner_radius: 4.0,
            padding: 8.0,
            gap: 4.0,
            child: None,
            child_node: None,
            element_id: ElementId::auto(),
            handlers: Rc::new(RefCell::new(EventHandlers::new())),
        }
    }

    /// Set tooltip position
    pub fn position(mut self, position: TooltipPosition) -> Self {
        self.position = position;
        self
    }

    /// Position above target
    pub fn top(self) -> Self {
        self.position(TooltipPosition::Top)
    }

    /// Position below target
    pub fn bottom(self) -> Self {
        self.position(TooltipPosition::Bottom)
    }

    /// Position left of target
    pub fn left(self) -> Self {
        self.position(TooltipPosition::Left)
    }

    /// Position right of target
    pub fn right(self) -> Self {
        self.position(TooltipPosition::Right)
    }

    /// Set background color
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }

    /// Set text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Set the child element that will trigger the tooltip
    pub fn child(mut self, child: impl Element + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }
}

impl Default for Tooltip {
    fn default() -> Self {
        Self::new("")
    }
}

impl Element for Tooltip {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        // Layout child
        if let Some(ref mut child) = self.child {
            let child_node = child.layout(ctx);
            self.child_node = Some(child_node);
            child_node
        } else {
            ctx.request_layout(Style::default())
        }
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        // Paint child first
        if let Some(ref mut child) = self.child {
            child.paint(bounds, ctx);
        }

        // Register for hover detection
        register_element(self.element_id, self.handlers.clone());
        ctx.register_hit_test(self.element_id, bounds, 0);

        // Check if hovered
        let state = get_element_state(self.element_id).unwrap_or_default();
        if !state.is_hovered {
            return;
        }

        // Measure tooltip text
        let text_style = TextStyle {
            size: 12.0,
            color: self.text_color,
        };
        let text_size = ctx.text_system.measure_text(
            &self.text,
            &crate::text_system::TextConfig {
                font_stack: parley::FontStack::from("system-ui"),
                size: text_style.size,
                weight: parley::FontWeight::NORMAL,
                color: text_style.color.clone(),
                line_height: 1.2,
            },
            Some(200.0), // Max width
            ctx.scale_factor,
        );

        // Calculate tooltip dimensions
        let tooltip_width = text_size.x + self.padding * 2.0;
        let tooltip_height = text_size.y + self.padding * 2.0;

        // Calculate tooltip position based on TooltipPosition
        let tooltip_pos = match self.position {
            TooltipPosition::Top => Vec2::new(
                bounds.pos.x + (bounds.size.x - tooltip_width) / 2.0,
                bounds.pos.y - tooltip_height - self.gap,
            ),
            TooltipPosition::Bottom => Vec2::new(
                bounds.pos.x + (bounds.size.x - tooltip_width) / 2.0,
                bounds.pos.y + bounds.size.y + self.gap,
            ),
            TooltipPosition::Left => Vec2::new(
                bounds.pos.x - tooltip_width - self.gap,
                bounds.pos.y + (bounds.size.y - tooltip_height) / 2.0,
            ),
            TooltipPosition::Right => Vec2::new(
                bounds.pos.x + bounds.size.x + self.gap,
                bounds.pos.y + (bounds.size.y - tooltip_height) / 2.0,
            ),
        };

        let tooltip_bounds = Rect::from_pos_size(tooltip_pos, Vec2::new(tooltip_width, tooltip_height));

        // Paint tooltip background (high z-index to appear on top)
        ctx.paint_quad(PaintQuad {
            bounds: tooltip_bounds,
            fill: self.background,
            corner_radii: Corners::all(self.corner_radius),
            border_widths: Edges::zero(),
            border_color: colors::TRANSPARENT,
        });

        // Paint tooltip text
        let text_pos = Vec2::new(
            tooltip_pos.x + self.padding,
            tooltip_pos.y + self.padding,
        );
        ctx.paint_text(PaintText {
            position: text_pos,
            text: self.text.clone(),
            style: text_style,
        });
    }
}
