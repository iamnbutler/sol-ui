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
}

impl HitTestEntry {
    pub fn new(element_id: ElementId, bounds: Rect, z_index: i32, layer_index: usize) -> Self {
        Self {
            element_id,
            bounds,
            z_index,
            layer_index,
        }
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
    pub fn new(layer_index: usize, z_base: i32) -> Self {
        Self {
            entries: Vec::new(),
            current_z_base: z_base,
            layer_index,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Point, Rect};
    use glam::Vec2;

    #[test]
    fn test_hit_test_entry_creation() {
        let entry = HitTestEntry::new(
            ElementId(42),
            Rect::new(10.0, 20.0, 100.0, 50.0),
            5,
            1,
        );

        assert_eq!(entry.element_id, ElementId(42));
        assert_eq!(entry.bounds, Rect::new(10.0, 20.0, 100.0, 50.0));
        assert_eq!(entry.z_index, 5);
        assert_eq!(entry.layer_index, 1);
    }

    #[test]
    fn test_hit_test_result_creation() {
        let result = HitTestResult {
            element_id: ElementId(123),
            bounds: Rect::new(0.0, 0.0, 200.0, 100.0),
            local_position: Vec2::new(50.0, 25.0),
            z_index: 10,
        };

        assert_eq!(result.element_id, ElementId(123));
        assert_eq!(result.bounds, Rect::new(0.0, 0.0, 200.0, 100.0));
        assert_eq!(result.local_position, Vec2::new(50.0, 25.0));
        assert_eq!(result.z_index, 10);
    }

    #[test]
    fn test_hit_test_builder_creation() {
        let builder = HitTestBuilder::new(2, 100);
        assert_eq!(builder.layer_index, 2);
        assert_eq!(builder.current_z_base, 100);
        assert!(builder.entries.is_empty());
    }

    #[test]
    fn test_hit_test_builder_add_entry() {
        let mut builder = HitTestBuilder::new(1, 50);
        let bounds = Rect::new(10.0, 10.0, 50.0, 50.0);

        builder.add_entry(ElementId(1), bounds, 5);

        assert_eq!(builder.entries.len(), 1);
        let entry = &builder.entries[0];
        assert_eq!(entry.element_id, ElementId(1));
        assert_eq!(entry.bounds, bounds);
        assert_eq!(entry.z_index, 55); // base (50) + relative (5)
        assert_eq!(entry.layer_index, 1);
    }

    #[test]
    fn test_hit_test_builder_z_context() {
        let mut builder = HitTestBuilder::new(0, 0);

        // Add entry at base z
        builder.add_entry(ElementId(1), Rect::new(0.0, 0.0, 10.0, 10.0), 0);
        assert_eq!(builder.entries.last().unwrap().z_index, 0);

        // Push z context
        builder.push_z_context(10);
        builder.add_entry(ElementId(2), Rect::new(0.0, 0.0, 10.0, 10.0), 5);
        assert_eq!(builder.entries.last().unwrap().z_index, 15); // 0 + 10 + 5

        // Push another context
        builder.push_z_context(20);
        builder.add_entry(ElementId(3), Rect::new(0.0, 0.0, 10.0, 10.0), 2);
        assert_eq!(builder.entries.last().unwrap().z_index, 32); // 0 + 10 + 20 + 2

        // Pop context
        builder.pop_z_context(20);
        builder.add_entry(ElementId(4), Rect::new(0.0, 0.0, 10.0, 10.0), 1);
        assert_eq!(builder.entries.last().unwrap().z_index, 11); // 0 + 10 + 1

        // Pop remaining context
        builder.pop_z_context(10);
        builder.add_entry(ElementId(5), Rect::new(0.0, 0.0, 10.0, 10.0), 3);
        assert_eq!(builder.entries.last().unwrap().z_index, 3); // 0 + 3
    }

    #[test]
    fn test_hit_test_builder_build_sorting() {
        let mut builder = HitTestBuilder::new(0, 0);

        // Add entries with different z-indices (out of order)
        builder.add_entry(ElementId(1), Rect::new(0.0, 0.0, 10.0, 10.0), 5);
        builder.add_entry(ElementId(2), Rect::new(0.0, 0.0, 10.0, 10.0), 10);
        builder.add_entry(ElementId(3), Rect::new(0.0, 0.0, 10.0, 10.0), 2);

        let sorted_entries = builder.build();

        // Should be sorted by z-index in descending order (highest first)
        assert_eq!(sorted_entries[0].element_id, ElementId(2)); // z=10
        assert_eq!(sorted_entries[1].element_id, ElementId(1)); // z=5
        assert_eq!(sorted_entries[2].element_id, ElementId(3)); // z=2
    }

    #[test]
    fn test_hit_test_builder_build_sorting_with_layers() {
        let mut builder1 = HitTestBuilder::new(0, 0); // Layer 0
        let mut builder2 = HitTestBuilder::new(1, 0); // Layer 1

        // Add entries with same z-index but different layers
        builder1.add_entry(ElementId(1), Rect::new(0.0, 0.0, 10.0, 10.0), 5);
        builder2.add_entry(ElementId(2), Rect::new(0.0, 0.0, 10.0, 10.0), 5);

        // Combine entries for testing
        let mut all_entries = builder1.build();
        all_entries.extend(builder2.build());

        // Sort manually to test the sorting behavior
        all_entries.sort_by(|a, b| {
            b.z_index
                .cmp(&a.z_index)
                .then_with(|| b.layer_index.cmp(&a.layer_index))
        });

        // Higher layer index should come first when z-index is equal
        assert_eq!(all_entries[0].element_id, ElementId(2)); // layer 1
        assert_eq!(all_entries[1].element_id, ElementId(1)); // layer 0
    }

    #[test]
    fn test_hit_test_builder_multiple_entries() {
        let mut builder = HitTestBuilder::new(0, 0);

        // Add multiple entries
        for i in 0..5 {
            builder.add_entry(
                ElementId(i),
                Rect::new(i as f32 * 10.0, 0.0, 10.0, 10.0),
                i as i32,
            );
        }

        assert_eq!(builder.entries.len(), 5);

        let sorted_entries = builder.build();
        assert_eq!(sorted_entries.len(), 5);

        // Should be sorted by z-index descending
        for i in 0..5 {
            assert_eq!(sorted_entries[i].element_id, ElementId(4 - i));
        }
    }

    #[test]
    fn test_hit_testable_trait_defaults() {
        struct TestElement(ElementId);

        impl HitTestable for TestElement {
            fn element_id(&self) -> ElementId {
                self.0
            }
        }

        let element = TestElement(ElementId(42));
        assert_eq!(element.element_id(), ElementId(42));
        assert!(element.is_interactive()); // default should be true
        assert_eq!(element.z_index_offset(), 0); // default should be 0
    }

    #[test]
    fn test_hit_testable_trait_custom_implementations() {
        struct CustomElement {
            id: ElementId,
            interactive: bool,
            z_offset: i32,
        }

        impl HitTestable for CustomElement {
            fn element_id(&self) -> ElementId {
                self.id
            }

            fn is_interactive(&self) -> bool {
                self.interactive
            }

            fn z_index_offset(&self) -> i32 {
                self.z_offset
            }
        }

        let interactive_element = CustomElement {
            id: ElementId(1),
            interactive: true,
            z_offset: 10,
        };
        assert!(interactive_element.is_interactive());
        assert_eq!(interactive_element.z_index_offset(), 10);

        let non_interactive_element = CustomElement {
            id: ElementId(2),
            interactive: false,
            z_offset: -5,
        };
        assert!(!non_interactive_element.is_interactive());
        assert_eq!(non_interactive_element.z_index_offset(), -5);
    }
}
