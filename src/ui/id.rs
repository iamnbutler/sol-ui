use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A unique identifier for a widget in the immediate mode UI system.
///
/// IDs are generated based on the widget's location in the code and optional user data.
/// This allows widgets to maintain state across frames while being created dynamically.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create a new widget ID from a source location.
    /// This is typically used with the `widget_id!()` macro.
    pub fn from_source_location(file: &str, line: u32, column: u32) -> Self {
        let mut hasher = DefaultHasher::new();
        file.hash(&mut hasher);
        line.hash(&mut hasher);
        column.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Create a widget ID with additional user data.
    /// Useful for widgets in loops or dynamic lists.
    pub fn with_data<T: Hash>(self, data: T) -> Self {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        data.hash(&mut hasher);
        Self(hasher.finish())
    }

    /// Combine this ID with a child index.
    /// Used internally for nested widgets.
    pub fn with_index(self, index: usize) -> Self {
        self.with_data(index)
    }

    /// Get the raw hash value.
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// Stack-based ID generation for hierarchical widgets.
pub struct IdStack {
    stack: Vec<WidgetId>,
    current_child_index: Vec<usize>,
}

impl IdStack {
    pub fn new() -> Self {
        Self {
            stack: vec![WidgetId(0)], // Root ID
            current_child_index: vec![0],
        }
    }

    /// Push a new ID onto the stack.
    pub fn push(&mut self, base_id: WidgetId) {
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
    pub fn current(&self) -> WidgetId {
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

/// Macro to generate a widget ID from the current source location.
#[macro_export]
macro_rules! widget_id {
    () => {
        $crate::ui::WidgetId::from_source_location(file!(), line!(), column!())
    };
    ($data:expr) => {
        $crate::ui::WidgetId::from_source_location(file!(), line!(), column!()).with_data($data)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_id_equality() {
        let id1 = WidgetId::from_source_location("test.rs", 10, 5);
        let id2 = WidgetId::from_source_location("test.rs", 10, 5);
        let id3 = WidgetId::from_source_location("test.rs", 11, 5);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_widget_id_with_data() {
        let base = WidgetId::from_source_location("test.rs", 10, 5);
        let id1 = base.with_data(1);
        let id2 = base.with_data(2);

        assert_ne!(id1, id2);
        assert_ne!(id1, base);
    }

    #[test]
    fn test_id_stack() {
        let mut stack = IdStack::new();
        let id1 = WidgetId::from_source_location("test.rs", 10, 5);

        stack.push(id1);
        let stacked_id1 = stack.current();

        stack.push(id1); // Same base ID but different position in hierarchy
        let stacked_id2 = stack.current();

        assert_ne!(stacked_id1, stacked_id2);
    }
}
