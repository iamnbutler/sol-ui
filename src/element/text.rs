use crate::{
    element::{Element, LayoutContext, PaintContext},
    geometry::Rect,
    layout_id::LayoutId,
    render::PaintText,
    style::TextStyle,
};
use taffy::prelude::*;

/// Create a new text element
pub fn text(content: impl Into<String>, style: TextStyle) -> Text {
    Text::new(content, style)
}

/// A simple text element
pub struct Text {
    content: String,
    style: TextStyle,
    node_id: Option<NodeId>,
    /// Stable layout ID for caching across frames
    layout_id: Option<LayoutId>,
}

impl Text {
    pub fn new(content: impl Into<String>, style: TextStyle) -> Self {
        Self {
            content: content.into(),
            style,
            node_id: None,
            layout_id: None,
        }
    }

    /// Set a stable layout ID for caching across frames.
    pub fn layout_id(mut self, id: impl Into<LayoutId>) -> Self {
        self.layout_id = Some(id.into());
        self
    }
}

impl Element for Text {
    fn layout(&mut self, ctx: &mut LayoutContext) -> NodeId {
        let node_id = if let Some(ref layout_id) = self.layout_id {
            // Use cached layout
            ctx.request_text_layout_cached(layout_id, Style::default(), &self.content, &self.style)
        } else {
            // Immediate mode
            ctx.request_text_layout(Style::default(), &self.content, &self.style)
        };
        self.node_id = Some(node_id);
        node_id
    }

    fn paint(&mut self, bounds: Rect, ctx: &mut PaintContext) {
        if !ctx.is_visible(&bounds) {
            return;
        }

        ctx.paint_text(PaintText {
            position: bounds.pos,
            text: self.content.clone(),
            style: self.style.clone(),
            measured_size: Some(bounds.size),
        });
    }
}
