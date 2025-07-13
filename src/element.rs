//! Two-phase element rendering system
//!
mod container;
mod text;

pub use container::{Container, column, container, row};
pub use text::{Text, text};

use crate::{
    color::Color,
    geometry::{Edges, Rect},
    interaction::{ElementId, hit_test::HitTestBuilder},
    layout_engine::{ElementData, TaffyLayoutEngine},
    render::{DrawList, PaintQuad, PaintShadow, PaintText},
    style::TextStyle,
    text_system::TextSystem,
};
use glam::Vec2;
use std::cell::RefCell;
use std::rc::Rc;
use taffy::prelude::*;

/// Elements participate in a two-phase rendering process
pub trait Element {
    /// Phase 1: Declare layout requirements and return a layout id
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId;

    /// Phase 2: Paint using the computed bounds
    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext);
}

/// Context for the layout phase
pub struct LayoutContext<'a> {
    pub(crate) engine: &'a mut TaffyLayoutEngine,
    pub(crate) text_system: &'a mut TextSystem,
    pub(crate) scale_factor: f32,
}

impl<'a> LayoutContext<'a> {
    /// Request layout for a leaf node (no children)
    pub fn request_layout(&mut self, style: Style) -> NodeId {
        self.engine.request_layout(style, &[])
    }

    /// Request layout with children
    pub fn request_layout_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        self.engine.request_layout(style, children)
    }

    /// Request layout for a text element that needs measuring
    pub fn request_text_layout(
        &mut self,
        style: Style,
        text: &str,
        text_style: &TextStyle,
    ) -> NodeId {
        // Store text data for measurement
        let data = ElementData {
            text: Some((text.to_string(), text_style.clone())),
            background: None,
        };
        self.engine.request_layout_with_data(style, data, &[])
    }

    /// Request layout with custom data
    pub fn request_layout_with_data(
        &mut self,
        style: Style,
        data: ElementData,
        children: &[NodeId],
    ) -> NodeId {
        self.engine.request_layout_with_data(style, data, children)
    }

    /// Measure text (for use during layout)
    pub fn measure_text(&mut self, text: &str, style: &TextStyle, max_width: Option<f32>) -> Vec2 {
        let text_config = crate::text_system::TextConfig {
            font_stack: parley::FontStack::from("system-ui"),
            size: style.size,
            weight: parley::FontWeight::NORMAL,
            color: style.color.clone(),
            line_height: 1.2,
        };

        self.text_system
            .measure_text(text, &text_config, max_width, self.scale_factor)
    }
}

/// Context for the paint phase
pub struct PaintContext<'a> {
    pub(crate) draw_list: &'a mut DrawList,
    pub(crate) text_system: &'a mut TextSystem,
    pub(crate) layout_engine: &'a TaffyLayoutEngine,
    pub(crate) scale_factor: f32,
    pub(crate) parent_offset: Vec2,
    pub(crate) hit_test_builder: Option<Rc<RefCell<HitTestBuilder>>>,
}

impl<'a> PaintContext<'a> {
    /// Paint a quad with all its properties
    pub fn paint_quad(&mut self, quad: PaintQuad) {
        // For now, just handle the fill
        // TODO: Handle borders, corner radii, etc.
        self.draw_list.add_rect(quad.bounds, quad.fill);

        // Paint borders if present
        if quad.border_widths != Edges::zero()
            && quad.border_color != crate::color::colors::TRANSPARENT
        {
            // Top edge
            if quad.border_widths.top > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos,
                        Vec2::new(quad.bounds.size.x, quad.border_widths.top),
                    ),
                    quad.border_color,
                );
            }

            // Bottom edge
            if quad.border_widths.bottom > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos
                            + Vec2::new(0.0, quad.bounds.size.y - quad.border_widths.bottom),
                        Vec2::new(quad.bounds.size.x, quad.border_widths.bottom),
                    ),
                    quad.border_color,
                );
            }

            // Left edge
            if quad.border_widths.left > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos,
                        Vec2::new(quad.border_widths.left, quad.bounds.size.y),
                    ),
                    quad.border_color,
                );
            }

            // Right edge
            if quad.border_widths.right > 0.0 {
                self.draw_list.add_rect(
                    Rect::from_pos_size(
                        quad.bounds.pos
                            + Vec2::new(quad.bounds.size.x - quad.border_widths.right, 0.0),
                        Vec2::new(quad.border_widths.right, quad.bounds.size.y),
                    ),
                    quad.border_color,
                );
            }
        }
    }

    /// Paint text
    pub fn paint_text(&mut self, text: PaintText) {
        self.draw_list
            .add_text(text.position, &text.text, text.style);
    }

    /// Paint a shadow
    pub fn paint_shadow(&mut self, _shadow: PaintShadow) {
        // TODO: Add shadow support to draw list
        // For now this is a no-op
    }

    /// Helper to create a simple filled quad
    pub fn paint_solid_quad(&mut self, bounds: Rect, color: Color) {
        self.paint_quad(PaintQuad::filled(bounds, color));
    }

    /// Check if a rect is visible (for culling)
    pub fn is_visible(&self, rect: &Rect) -> bool {
        if let Some(viewport) = self.draw_list.viewport() {
            viewport.intersect(rect).is_some()
        } else {
            true
        }
    }

    /// Get the computed bounds for a node
    pub fn get_bounds(&self, node_id: NodeId) -> Rect {
        let local_bounds = self.layout_engine.layout_bounds(node_id);
        Rect::from_pos_size(self.parent_offset + local_bounds.pos, local_bounds.size)
    }

    /// Create a child paint context with updated offset
    pub fn child_context(&mut self, offset: Vec2) -> PaintContext {
        PaintContext {
            draw_list: self.draw_list,
            text_system: self.text_system,
            layout_engine: self.layout_engine,
            scale_factor: self.scale_factor,
            parent_offset: self.parent_offset + offset,
            hit_test_builder: self.hit_test_builder.clone(),
        }
    }

    /// Register an element for hit testing
    pub fn register_hit_test(&mut self, element_id: ElementId, bounds: Rect, z_index: i32) {
        if let Some(builder) = &self.hit_test_builder {
            // bounds are already in screen coordinates (absolute position)
            builder.borrow_mut().add_entry(element_id, bounds, z_index);
        }
    }
}
