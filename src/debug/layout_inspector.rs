//! Layout inspector for debugging Taffy layout

use crate::{
    color::{Color, ColorExt, colors},
    geometry::Rect,
    render::{PaintContext, PaintText},
    style::TextStyle,
};
use glam::Vec2;

/// Information about a layout node
#[derive(Debug, Clone)]
pub struct LayoutNodeInfo {
    pub node_id: u64,
    pub bounds: Rect,
    pub computed_size: Vec2,
    pub computed_position: Vec2,
    pub flex_direction: Option<String>,
    pub justify_content: Option<String>,
    pub align_items: Option<String>,
    pub padding: Option<[f32; 4]>,
    pub margin: Option<[f32; 4]>,
    pub gap: Option<f32>,
    pub children_count: usize,
    pub depth: usize,
}

/// Layout inspector for visualizing Taffy layout tree
pub struct LayoutInspector {
    nodes: Vec<LayoutNodeInfo>,
    selected_node: Option<u64>,
    show_tree: bool,
    show_details: bool,
}

impl LayoutInspector {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            selected_node: None,
            show_tree: true,
            show_details: true,
        }
    }

    /// Register a layout node
    pub fn register_node(&mut self, info: LayoutNodeInfo) {
        self.nodes.push(info);
    }

    /// Clear all nodes
    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Select a node for detailed inspection
    pub fn select_node(&mut self, node_id: Option<u64>) {
        self.selected_node = node_id;
    }

    /// Get the selected node
    pub fn selected_node(&self) -> Option<u64> {
        self.selected_node
    }

    /// Find node at position
    pub fn find_node_at(&self, position: Vec2) -> Option<&LayoutNodeInfo> {
        // Find the deepest (most nested) node containing the position
        let mut best_match: Option<&LayoutNodeInfo> = None;
        let mut best_depth = 0;

        for node in &self.nodes {
            if node.bounds.contains(crate::geometry::Point::new(position.x, position.y)) {
                if best_match.is_none() || node.depth > best_depth {
                    best_match = Some(node);
                    best_depth = node.depth;
                }
            }
        }

        best_match
    }

    /// Toggle tree display
    pub fn toggle_tree(&mut self) {
        self.show_tree = !self.show_tree;
    }

    /// Toggle details display
    pub fn toggle_details(&mut self) {
        self.show_details = !self.show_details;
    }

    /// Paint the layout inspector panel
    pub fn paint(&self, viewport: Rect, ctx: &mut PaintContext) {
        if self.nodes.is_empty() {
            return;
        }

        // Paint layout tree panel on the left
        if self.show_tree {
            self.paint_tree_panel(viewport, ctx);
        }

        // Paint details panel for selected node
        if self.show_details {
            if let Some(node_id) = self.selected_node {
                if let Some(node) = self.nodes.iter().find(|n| n.node_id == node_id) {
                    self.paint_details_panel(node, viewport, ctx);
                }
            }
        }
    }

    fn paint_tree_panel(&self, viewport: Rect, ctx: &mut PaintContext) {
        let panel_width = 200.0;
        let panel_bounds = Rect::from_pos_size(
            viewport.pos,
            Vec2::new(panel_width, viewport.size.y.min(400.0)),
        );

        // Background
        ctx.paint_solid_quad(panel_bounds, Color::rgba(0.1, 0.1, 0.1, 0.9));

        // Title
        ctx.paint_text(PaintText {
            position: panel_bounds.pos + Vec2::new(8.0, 8.0),
            text: "Layout Tree".to_string(),
            style: TextStyle {
                size: 12.0,
                color: colors::WHITE,
                line_height: 1.2,
            },
        });

        // Tree items
        let mut y = 28.0;
        let line_height = 16.0;

        for node in &self.nodes {
            if y > panel_bounds.size.y - line_height {
                break;
            }

            let indent = node.depth as f32 * 12.0;
            let is_selected = self.selected_node == Some(node.node_id);

            // Highlight selected
            if is_selected {
                ctx.paint_solid_quad(
                    Rect::from_pos_size(
                        panel_bounds.pos + Vec2::new(4.0, y - 2.0),
                        Vec2::new(panel_width - 8.0, line_height),
                    ),
                    Color::rgba(0.2, 0.4, 0.8, 0.5),
                );
            }

            // Node label
            let label = format!(
                "{}#{} ({:.0}x{:.0})",
                if node.children_count > 0 { "+" } else { "-" },
                node.node_id,
                node.bounds.size.x,
                node.bounds.size.y
            );

            ctx.paint_text(PaintText {
                position: panel_bounds.pos + Vec2::new(8.0 + indent, y),
                text: label,
                style: TextStyle {
                    size: 10.0,
                    color: if is_selected {
                        colors::WHITE
                    } else {
                        Color::rgba(0.8, 0.8, 0.8, 1.0)
                    },
                    line_height: 1.2,
                },
            });

            y += line_height;
        }
    }

    fn paint_details_panel(&self, node: &LayoutNodeInfo, viewport: Rect, ctx: &mut PaintContext) {
        let panel_width = 220.0;
        let panel_height = 200.0;
        let panel_bounds = Rect::from_pos_size(
            viewport.pos + Vec2::new(viewport.size.x - panel_width - 8.0, 8.0),
            Vec2::new(panel_width, panel_height),
        );

        // Background
        ctx.paint_solid_quad(panel_bounds, Color::rgba(0.1, 0.1, 0.1, 0.9));

        // Title
        ctx.paint_text(PaintText {
            position: panel_bounds.pos + Vec2::new(8.0, 8.0),
            text: format!("Node #{}", node.node_id),
            style: TextStyle {
                size: 12.0,
                color: colors::WHITE,
                line_height: 1.2,
            },
        });

        // Details
        let mut y = 28.0;
        let line_height = 14.0;

        let details = [
            format!("Position: ({:.0}, {:.0})", node.bounds.pos.x, node.bounds.pos.y),
            format!("Size: {:.0} x {:.0}", node.bounds.size.x, node.bounds.size.y),
            format!("Depth: {}", node.depth),
            format!("Children: {}", node.children_count),
            node.flex_direction.clone().map_or(String::new(), |d| format!("Direction: {}", d)),
            node.justify_content.clone().map_or(String::new(), |j| format!("Justify: {}", j)),
            node.align_items.clone().map_or(String::new(), |a| format!("Align: {}", a)),
            node.padding.map_or(String::new(), |p| format!("Padding: [{:.0},{:.0},{:.0},{:.0}]", p[0], p[1], p[2], p[3])),
            node.margin.map_or(String::new(), |m| format!("Margin: [{:.0},{:.0},{:.0},{:.0}]", m[0], m[1], m[2], m[3])),
            node.gap.map_or(String::new(), |g| format!("Gap: {:.0}", g)),
        ];

        for detail in details {
            if detail.is_empty() {
                continue;
            }

            ctx.paint_text(PaintText {
                position: panel_bounds.pos + Vec2::new(8.0, y),
                text: detail,
                style: TextStyle {
                    size: 10.0,
                    color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                    line_height: 1.2,
                },
            });

            y += line_height;
        }
    }
}

impl Default for LayoutInspector {
    fn default() -> Self {
        Self::new()
    }
}
