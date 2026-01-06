//! Layout engine using Taffy for flexbox/grid layout

use crate::geometry::Rect;
use crate::layout_id::LayoutId;
use glam::Vec2;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use taffy::prelude::*;
use tracing::{debug, info_span};

/// Data stored with each element in the taffy tree
#[derive(Debug, Clone, Default)]
pub struct ElementData {
    pub text: Option<(String, crate::style::TextStyle)>,
    pub background: Option<crate::color::Color>,
}

/// Cached node information for retained-mode layout
#[derive(Debug)]
struct CachedNode {
    node_id: NodeId,
    style_hash: u64,
    children_hash: u64,
    data_hash: u64,
}

/// Cache for reusing Taffy nodes across frames
#[derive(Debug, Default)]
pub struct LayoutCache {
    /// Maps LayoutId to cached node info
    nodes: HashMap<LayoutId, CachedNode>,
    /// LayoutIds that were used this frame
    live_this_frame: HashSet<LayoutId>,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Begin a new frame - clear the live set but keep cached nodes
    pub fn begin_frame(&mut self) {
        self.live_this_frame.clear();
    }

    /// Mark a layout ID as live this frame
    pub fn mark_live(&mut self, id: &LayoutId) {
        self.live_this_frame.insert(id.clone());
    }

    /// Check if we have a cached node for this ID
    fn get(&self, id: &LayoutId) -> Option<&CachedNode> {
        self.nodes.get(id)
    }

    /// Insert or update a cached node
    fn insert(&mut self, id: LayoutId, node: CachedNode) {
        self.nodes.insert(id, node);
    }

    /// End frame - remove nodes that weren't used
    pub fn end_frame(&mut self, taffy: &mut TaffyTree<ElementData>) {
        let dead_ids: Vec<LayoutId> = self
            .nodes
            .keys()
            .filter(|id| !self.live_this_frame.contains(*id))
            .cloned()
            .collect();

        for id in &dead_ids {
            if let Some(cached) = self.nodes.remove(id) {
                // Remove from taffy tree
                let _ = taffy.remove(cached.node_id);
                debug!("Removed dead layout node: {}", id);
            }
        }

        if !dead_ids.is_empty() {
            debug!("Cleaned up {} dead layout nodes", dead_ids.len());
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.nodes.len(), self.live_this_frame.len())
    }
}

/// Compute a hash for a Taffy Style
fn hash_style(style: &Style) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();

    // Hash display and position
    std::mem::discriminant(&style.display).hash(&mut hasher);
    std::mem::discriminant(&style.position).hash(&mut hasher);

    // Hash flex properties
    std::mem::discriminant(&style.flex_direction).hash(&mut hasher);
    std::mem::discriminant(&style.flex_wrap).hash(&mut hasher);
    style.flex_grow.to_bits().hash(&mut hasher);
    style.flex_shrink.to_bits().hash(&mut hasher);

    // Hash alignment
    std::mem::discriminant(&style.align_items).hash(&mut hasher);
    std::mem::discriminant(&style.align_self).hash(&mut hasher);
    std::mem::discriminant(&style.align_content).hash(&mut hasher);
    std::mem::discriminant(&style.justify_content).hash(&mut hasher);
    std::mem::discriminant(&style.justify_items).hash(&mut hasher);
    std::mem::discriminant(&style.justify_self).hash(&mut hasher);

    // Hash size using debug format (covers all dimension variants)
    format!("{:?}", style.size).hash(&mut hasher);
    format!("{:?}", style.min_size).hash(&mut hasher);
    format!("{:?}", style.max_size).hash(&mut hasher);

    // Hash spacing
    format!("{:?}", style.padding).hash(&mut hasher);
    format!("{:?}", style.margin).hash(&mut hasher);
    format!("{:?}", style.border).hash(&mut hasher);
    format!("{:?}", style.gap).hash(&mut hasher);

    // Hash overflow
    format!("{:?}", style.overflow).hash(&mut hasher);

    hasher.finish()
}

/// Compute a hash for a list of child LayoutIds
fn hash_children(children: &[LayoutId]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for child in children {
        child.hash(&mut hasher);
    }
    hasher.finish()
}

/// Compute a hash for ElementData
fn hash_data(data: &ElementData) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    if let Some((text, style)) = &data.text {
        text.hash(&mut hasher);
        format!("{:?}", style).hash(&mut hasher);
    }
    if let Some(bg) = &data.background {
        format!("{:?}", bg).hash(&mut hasher);
    }
    hasher.finish()
}

/// A layout engine that wraps Taffy and provides a simple API
pub struct TaffyLayoutEngine {
    taffy: TaffyTree<ElementData>,
    cache: LayoutCache,
}

impl TaffyLayoutEngine {
    /// Create a new layout engine
    pub fn new() -> Self {
        TaffyLayoutEngine {
            taffy: TaffyTree::new(),
            cache: LayoutCache::new(),
        }
    }

    /// Begin a new frame - prepares for layout but doesn't clear cached nodes
    pub fn begin_frame(&mut self) {
        self.cache.begin_frame();
    }

    /// End frame - cleans up nodes that weren't used this frame
    pub fn end_frame(&mut self) {
        self.cache.end_frame(&mut self.taffy);
    }

    /// Clear all layout data (for non-cached elements)
    ///
    /// Note: This clears the entire tree including cached nodes.
    /// Prefer using begin_frame/end_frame for retained-mode layout.
    pub fn clear(&mut self) {
        self.taffy.clear();
        self.cache = LayoutCache::new();
    }

    /// Request layout for a node with a stable ID (cached/retained mode)
    ///
    /// If the node exists in cache and hasn't changed, reuses it.
    /// Otherwise creates or updates the node.
    pub fn request_layout_cached(
        &mut self,
        layout_id: &LayoutId,
        style: Style,
        data: ElementData,
        child_ids: &[LayoutId],
        child_nodes: &[NodeId],
    ) -> NodeId {
        let style_hash = hash_style(&style);
        let children_hash = hash_children(child_ids);
        let data_hash = hash_data(&data);

        self.cache.mark_live(layout_id);

        if let Some(cached) = self.cache.get(layout_id) {
            // Check if anything changed
            if cached.style_hash == style_hash
                && cached.children_hash == children_hash
                && cached.data_hash == data_hash
            {
                // Nothing changed - reuse existing node
                debug!("Reusing cached layout node: {}", layout_id);
                return cached.node_id;
            }

            // Something changed - update the existing node
            debug!("Updating cached layout node: {}", layout_id);
            let node_id = cached.node_id;

            // Update style
            self.taffy
                .set_style(node_id, style)
                .expect("Failed to set style");

            // Update children
            self.taffy
                .set_children(node_id, child_nodes)
                .expect("Failed to set children");

            // Update data
            if let Some(ctx) = self.taffy.get_node_context_mut(node_id) {
                *ctx = data;
            }

            // Mark dirty for re-layout
            self.taffy
                .mark_dirty(node_id)
                .expect("Failed to mark dirty");

            // Update cache entry
            self.cache.insert(
                layout_id.clone(),
                CachedNode {
                    node_id,
                    style_hash,
                    children_hash,
                    data_hash,
                },
            );

            return node_id;
        }

        // No cached node - create new one
        debug!("Creating new cached layout node: {}", layout_id);
        let node_id = if child_nodes.is_empty() {
            self.taffy
                .new_leaf_with_context(style, data)
                .expect("Failed to create leaf node")
        } else {
            let node = self
                .taffy
                .new_leaf_with_context(style, data)
                .expect("Failed to create node");
            for &child in child_nodes {
                self.taffy
                    .add_child(node, child)
                    .expect("Failed to add child");
            }
            node
        };

        self.cache.insert(
            layout_id.clone(),
            CachedNode {
                node_id,
                style_hash,
                children_hash,
                data_hash,
            },
        );

        node_id
    }

    /// Request layout for a leaf node (no children) - immediate mode
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

    /// Request layout with associated data - immediate mode
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

    /// Get mutable access to the underlying taffy tree (for testing)
    #[cfg(any(test, feature = "testing"))]
    pub fn taffy_mut(&mut self) -> &mut TaffyTree<ElementData> {
        &mut self.taffy
    }

    /// Get cache statistics (total cached, live this frame)
    pub fn cache_stats(&self) -> (usize, usize) {
        self.cache.stats()
    }
}

impl Default for TaffyLayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Measure function for elements that contain text
fn measure_element(
    known_dimensions: Size<Option<f32>>,
    available_space: Size<AvailableSpace>,
    node_data: Option<&mut ElementData>,
    text_system: &mut crate::text_system::TextSystem,
    scale_factor: f32,
) -> Size<f32> {
    // Use known dimensions if available (optimization)
    if let (Some(width), Some(height)) = (known_dimensions.width, known_dimensions.height) {
        return Size { width, height };
    }

    if let Some(data) = node_data {
        if let Some((content, style)) = &data.text {
            let max_width = match available_space.width {
                AvailableSpace::Definite(w) => Some(w),
                _ => known_dimensions.width,
            };

            let text_config = crate::text_system::TextConfig {
                font_stack: parley::FontStack::from(style.font_family),
                size: style.size,
                weight: style.weight,
                color: style.color.clone(),
                line_height: style.line_height,
            };

            let measured_size =
                text_system.measure_text(content, &text_config, max_width, scale_factor);

            Size {
                width: known_dimensions.width.unwrap_or(measured_size.x),
                height: known_dimensions.height.unwrap_or(measured_size.y),
            }
        } else {
            Size::ZERO
        }
    } else {
        Size::ZERO
    }
}
