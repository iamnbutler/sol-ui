//! Two-phase element rendering system
//!
mod container;
mod text;

pub use container::{Container, column, container, row};
pub use text::{Text, text};

use crate::{
    geometry::Rect,
    layout_engine::{ElementData, TaffyLayoutEngine},
    render::PaintContext,
    style::TextStyle,
    text_system::TextSystem,
};
use glam::Vec2;
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
