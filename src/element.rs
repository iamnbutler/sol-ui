//! Element identification and hierarchy management

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A unique identifier for an element in the UI system.
///
/// IDs are generated based on the element's location in the code and optional user data.
/// This allows elements to maintain state across frames while being created dynamically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ElementId(u64);

impl ElementId {
    /// Create a new element ID from a source location.
    /// This is typically used with the `element_id!()` macro.
    pub fn from_source_location(file: &str, line: u32, column: u32) -> Self {
        let mut hasher = DefaultHasher::new();
        file.hash(&mut hasher);
        line.hash(&mut hasher);
        column.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Create an element ID with additional user data.
    /// Useful for elements in loops or dynamic lists.
    pub fn with_data<T: Hash>(self, data: T) -> Self {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        data.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Combine this ID with a child index.
    /// Used internally for nested elements.
    pub fn with_index(self, index: usize) -> Self {
        self.with_data(index)
    }

    /// Get the raw hash value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Stack-based ID generation for hierarchical elements.
pub struct IdStack {
    stack: Vec<ElementId>,
    current_child_index: Vec<usize>,
}

impl IdStack {
    pub fn new() -> Self {
        Self {
            stack: vec![ElementId(0)], // Root ID
            current_child_index: vec![0],
        }
    }

    /// Push a new ID onto the stack.
    pub fn push(&mut self, base_id: ElementId) {
        let parent = self.stack.last().unwrap();
        let child_index = self.current_child_index.last_mut().unwrap();

        // Combine parent ID with child index and base ID
        let combined_id = parent.with_index(*child_index).with_data(base_id.0);

        *child_index += 1;

        self.stack.push(combined_id);
        self.current_child_index.push(0);
    }

    /// Pop the current ID from the stack.
    pub fn pop(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
            self.current_child_index.pop();
        }
    }

    /// Get the current ID.
    pub fn current(&self) -> ElementId {
        *self.stack.last().unwrap()
    }

    /// Reset child counter for the current level.
    /// Called at the start of each frame for each level.
    pub fn reset_child_counter(&mut self) {
        if let Some(counter) = self.current_child_index.last_mut() {
            *counter = 0;
        }
    }
}

/// Macro to generate an element ID from the current source location.
#[macro_export]
macro_rules! element_id {
    () => {
        $crate::element::ElementId::from_source_location(file!(), line!(), column!())
    };
    ($data:expr) => {
        $crate::element::ElementId::from_source_location(file!(), line!(), column!())
            .with_data($data)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_id_equality() {
        let id1 = ElementId::from_source_location("test.rs", 10, 5);
        let id2 = ElementId::from_source_location("test.rs", 10, 5);
        let id3 = ElementId::from_source_location("test.rs", 11, 5);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_element_id_with_data() {
        let base = ElementId::from_source_location("test.rs", 10, 5);
        let id1 = base.with_data(1);
        let id2 = base.with_data(2);

        assert_ne!(id1, id2);
        assert_ne!(id1, base);
    }

    #[test]
    fn test_id_stack() {
        let mut stack = IdStack::new();
        let id1 = ElementId::from_source_location("test.rs", 10, 5);

        stack.push(id1);
        let stacked_id1 = stack.current();

        stack.push(id1); // Same base ID but different position in hierarchy
        let stacked_id2 = stack.current();

        assert_ne!(stacked_id1, stacked_id2);
    }

    #[test]
    fn test_id_stack_hierarchy() {
        let mut stack = IdStack::new();

        // First child
        let id1 = ElementId::from_source_location("test.rs", 10, 5);
        stack.push(id1);
        let first_child = stack.current();

        // Second child at same level
        stack.pop();
        stack.push(id1);
        let second_child = stack.current();

        // These should be different because they have different child indices
        assert_ne!(first_child, second_child);
    }
}
