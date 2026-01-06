//! Lazy-initialized entity state for render closures
//!
//! This module provides `StateCell<T>`, a helper type that simplifies
//! the common pattern of creating entity state that persists across frames.

use super::{new_entity, Entity};
use std::cell::RefCell;

/// A cell for lazy-initialized entity state in render closures
///
/// This encapsulates the common pattern of creating entity state
/// outside a render closure and initializing it on first render.
///
/// # Example
/// ```ignore
/// use sol_ui::entity::{StateCell, observe};
///
/// let counter = StateCell::new();
///
/// layers.add_ui_layer(0, LayerOptions::default(), move || {
///     let count = counter.get_or_init(|| CounterState { value: 0 });
///     let value = count.observe(|s| s.value).unwrap_or(0);
///     // ... build UI
/// });
/// ```
pub struct StateCell<T: 'static> {
    inner: RefCell<Option<Entity<T>>>,
}

impl<T: 'static> StateCell<T> {
    /// Create a new empty state cell
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(None),
        }
    }

    /// Get the entity, creating it with the given initial value if needed
    ///
    /// On first call, creates a new entity with the value returned by `init`.
    /// On subsequent calls, returns the existing entity.
    ///
    /// # Panics
    /// Panics if called outside of a render context.
    pub fn get_or_init(&self, init: impl FnOnce() -> T) -> Entity<T> {
        let mut inner = self.inner.borrow_mut();
        if inner.is_none() {
            *inner = Some(new_entity(init()));
        }
        inner.clone().unwrap()
    }

    /// Check if the entity has been initialized
    pub fn is_init(&self) -> bool {
        self.inner.borrow().is_some()
    }

    /// Get the entity if it has been initialized
    pub fn get(&self) -> Option<Entity<T>> {
        self.inner.borrow().clone()
    }
}

impl<T: 'static> Default for StateCell<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{clear_entity_store, set_entity_store, EntityStore};

    struct TestState {
        value: i32,
    }

    #[test]
    fn test_state_cell_initialization() {
        let mut store = EntityStore::new();
        set_entity_store(&mut store);

        let cell: StateCell<TestState> = StateCell::new();

        assert!(!cell.is_init());
        assert!(cell.get().is_none());

        let entity = cell.get_or_init(|| TestState { value: 42 });
        assert!(cell.is_init());
        assert!(cell.get().is_some());

        // Second call returns same entity
        let entity2 = cell.get_or_init(|| TestState { value: 99 });
        assert_eq!(entity.id(), entity2.id());

        clear_entity_store();
    }

    #[test]
    fn test_state_cell_init_only_once() {
        let mut store = EntityStore::new();
        set_entity_store(&mut store);

        let cell: StateCell<TestState> = StateCell::new();
        let init_count = std::cell::Cell::new(0);

        // First call - should initialize
        let _ = cell.get_or_init(|| {
            init_count.set(init_count.get() + 1);
            TestState { value: 1 }
        });
        assert_eq!(init_count.get(), 1);

        // Second call - should NOT call init
        let _ = cell.get_or_init(|| {
            init_count.set(init_count.get() + 1);
            TestState { value: 2 }
        });
        assert_eq!(init_count.get(), 1);

        clear_entity_store();
    }
}
