//! Hit testing for interaction system

use super::ElementId;
use crate::geometry::Rect;

/// Entry in the hit test list
#[derive(Debug, Clone)]
pub struct HitTestEntry {
    /// The element's unique ID
    pub element_id: ElementId,

    /// The element's bounds in screen coordinates
    pub bounds: Rect,

    /// The element's z-index (higher values are on top)
    pub z_index: i32,

    /// Layer index this element belongs to
    pub layer_index: usize,

    /// Whether this element can receive keyboard focus
    pub focusable: bool,
}

impl HitTestEntry {
    pub fn new(element_id: ElementId, bounds: Rect, z_index: i32, layer_index: usize) -> Self {
        Self {
            element_id,
            bounds,
            z_index,
            layer_index,
            focusable: false,
        }
    }

    pub fn with_focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

/// Result of a hit test
#[derive(Debug, Clone)]
pub struct HitTestResult {
    /// The element that was hit
    pub element_id: ElementId,

    /// The element's bounds
    pub bounds: Rect,

    /// Position relative to the element's top-left corner
    pub local_position: glam::Vec2,

    /// The element's z-index
    pub z_index: i32,
}

/// Builder for collecting hit test entries during rendering
pub struct HitTestBuilder {
    entries: Vec<HitTestEntry>,
    current_z_base: i32,
    layer_index: usize,
}

impl HitTestBuilder {
    /// Create a new HitTestBuilder with specified layer index and z-base
    pub fn new(layer_index: usize, z_base: i32) -> Self {
        Self {
            entries: Vec::new(),
            current_z_base: z_base,
            layer_index,
        }
    }

    /// Create a new HitTestBuilder with defaults (for testing)
    pub fn default_for_testing() -> Self {
        Self {
            entries: Vec::new(),
            current_z_base: 0,
            layer_index: 0,
        }
    }

    /// Add a hit test entry
    pub fn add_entry(&mut self, element_id: ElementId, bounds: Rect, relative_z: i32) {
        let entry = HitTestEntry::new(
            element_id,
            bounds,
            self.current_z_base + relative_z,
            self.layer_index,
        );
        self.entries.push(entry);
    }

    /// Add a focusable hit test entry
    pub fn add_focusable_entry(&mut self, element_id: ElementId, bounds: Rect, relative_z: i32) {
        let entry = HitTestEntry::new(
            element_id,
            bounds,
            self.current_z_base + relative_z,
            self.layer_index,
        )
        .with_focusable(true);
        self.entries.push(entry);
    }

    /// Push a new z-index context (for nested elements)
    pub fn push_z_context(&mut self, z_offset: i32) {
        self.current_z_base += z_offset;
    }

    /// Pop z-index context
    pub fn pop_z_context(&mut self, z_offset: i32) {
        self.current_z_base -= z_offset;
    }

    /// Build the final sorted hit test list
    pub fn build(&mut self) -> Vec<HitTestEntry> {
        // Sort by z-index in descending order (highest z-index first)
        // This ensures we test top-most elements first
        self.entries.sort_by(|a, b| {
            b.z_index
                .cmp(&a.z_index)
                .then_with(|| b.layer_index.cmp(&a.layer_index))
        });
        self.entries.clone()
    }

    /// Get a reference to the current entries (for testing)
    pub fn entries(&self) -> &[HitTestEntry] {
        &self.entries
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

/// Trait for elements that participate in hit testing
pub trait HitTestable {
    /// Get the element's ID for hit testing
    fn element_id(&self) -> ElementId;

    /// Check if this element should participate in hit testing
    fn is_interactive(&self) -> bool {
        true
    }

    /// Get the z-index offset for this element (relative to parent)
    fn z_index_offset(&self) -> i32 {
        0
    }
}
