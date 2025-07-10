//! Layout engine using Taffy for flexbox/grid layout

use crate::geometry::Rect;
use glam::Vec2;
use std::collections::{HashMap, HashSet};
use taffy::prelude::*;
use tracing::info_span;

/// Data stored with each element in the taffy tree
#[derive(Debug, Clone, Default)]
pub struct ElementData {
    pub text: Option<(String, crate::draw::TextStyle)>,
    pub background: Option<crate::color::Color>,
}

/// A layout engine that wraps Taffy and provides a simple API
pub struct TaffyLayoutEngine {
    taffy: TaffyTree<ElementData>,
    absolute_layout_bounds: HashMap<NodeId, Rect>,
    computed_layouts: HashSet<NodeId>,
}

impl TaffyLayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        let taffy = TaffyTree::new();

        TaffyLayoutEngine {
            taffy,
            absolute_layout_bounds: HashMap::default(),
            computed_layouts: HashSet::default(),
        }
    }

    /// Clear all layout data (called every frame)
    pub fn clear(&mut self) {
        self.taffy.clear();
        self.absolute_layout_bounds.clear();
        self.computed_layouts.clear();
    }

    /// Request layout for a leaf node (no children)
    pub fn request_layout(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        if children.is_empty() {
            self.taffy
                .new_leaf(style)
                .expect("Failed to create leaf node")
        } else {
            self.taffy
                .new_with_children(style, children)
                .expect("Failed to create parent node")
        }
    }

    /// Request layout with associated data
    pub fn request_layout_with_data(
        &mut self,
        style: Style,
        data: ElementData,
        children: &[NodeId],
    ) -> NodeId {
        if children.is_empty() {
            self.taffy
                .new_leaf_with_context(style, data)
                .expect("Failed to create leaf node with data")
        } else {
            let node = self
                .taffy
                .new_leaf_with_context(style, data)
                .expect("Failed to create parent node with data");

            for &child in children {
                self.taffy
                    .add_child(node, child)
                    .expect("Failed to add child to node");
            }

            node
        }
    }

    /// Compute layout for the tree
    pub fn compute_layout(
        &mut self,
        root: NodeId,
        available_space: Size<AvailableSpace>,
        text_system: &mut crate::text_system::TextSystem,
        scale_factor: f32,
    ) -> Result<(), taffy::TaffyError> {
        let _compute_span = info_span!("compute_layout").entered();

        self.taffy.compute_layout_with_measure(
            root,
            available_space,
            |known_dimensions, available_space, _node_id, node_context, _style| {
                measure_element(
                    known_dimensions,
                    available_space,
                    node_context,
                    text_system,
                    scale_factor,
                )
            },
        )
    }

    /// Get the computed layout bounds for a node
    pub fn layout_bounds(&self, id: NodeId) -> Rect {
        let layout = self.taffy.layout(id).expect("Failed to get layout");
        Rect::from_pos_size(
            Vec2::new(layout.location.x, layout.location.y),
            Vec2::new(layout.size.width, layout.size.height),
        )
    }

    /// Get the element data for a node
    pub fn get_node_context(&self, id: NodeId) -> Option<&ElementData> {
        self.taffy.get_node_context(id)
    }

    /// Get the children of a node
    pub fn children(&self, id: NodeId) -> Result<Vec<NodeId>, taffy::TaffyError> {
        self.taffy.children(id)
    }
}

/// Measure function for elements that contain text
fn measure_element(
    _known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    node_data: Option<&mut ElementData>,
    text_system: &mut crate::text_system::TextSystem,
    scale_factor: f32,
) -> Size<f32> {
    if let Some(data) = node_data {
        if let Some((content, style)) = &data.text {
            let max_width = match available_space.width {
                AvailableSpace::Definite(w) => Some(w),
                _ => None,
            };

            let text_config = crate::text_system::TextConfig {
                font_stack: parley::FontStack::from("system-ui"),
                size: style.size,
                weight: parley::FontWeight::NORMAL,
                color: style.color.clone(),
                line_height: 1.2,
            };

            let measured_size =
                text_system.measure_text(content, &text_config, max_width, scale_factor);

            Size {
                width: measured_size.x,
                height: measured_size.y,
            }
        } else {
            Size::ZERO
        }
    } else {
        Size::ZERO
    }
}
