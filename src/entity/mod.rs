//! Entity system for persistent widget state
//!
//! This module provides GPUI-style entities - type-safe handles to state
//! that persists across frames, enabling stateful widgets while keeping
//! most UI stateless.
//!
//! ## Reactive Updates
//!
//! The entity system supports reactive updates through observation:
//!
//! - Use `observe(&entity, |state| ...)` to read state AND subscribe to changes
//! - When `update_entity` mutates observed state, the UI automatically re-renders
//! - Updates within a frame are batched to prevent excessive re-renders
//!
//! See the `subscription` module for details.

pub mod context;
pub mod derived;
pub mod store;
pub mod subscription;

pub use context::{
    clear_entity_store, new_entity, observe, read_entity, set_entity_store, update_entity,
    with_entity_store,
};
pub use derived::{derive, derive_from, derive_from2, Memo};
pub use store::EntityStore;
pub use subscription::SubscriptionManager;

use std::marker::PhantomData;

/// Unique identifier for an entity with generation for staleness detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId {
    /// Slot index in the entity store
    index: u32,
    /// Generation to detect use-after-free on slot reuse
    generation: u32,
}

impl EntityId {
    /// Create a new EntityId (internal use only)
    pub(crate) fn new(index: u32, generation: u32) -> Self {
        Self { index, generation }
    }

    /// Get the slot index
    pub(crate) fn index(&self) -> u32 {
        self.index
    }

    /// Get the generation
    pub(crate) fn generation(&self) -> u32 {
        self.generation
    }
}

/// Type-safe handle to persistent entity state
///
/// Entity<T> provides a handle to state of type T that lives in the EntityStore.
/// Multiple Entity<T> handles can point to the same state (reference counted).
/// When all handles are dropped, the state is cleaned up.
pub struct Entity<T: 'static> {
    id: EntityId,
    _marker: PhantomData<T>,
}

impl<T: 'static> Entity<T> {
    /// Create a new entity handle (internal use only)
    pub(crate) fn new(id: EntityId) -> Self {
        Self {
            id,
            _marker: PhantomData,
        }
    }

    /// Get the entity's ID
    pub fn id(&self) -> EntityId {
        self.id
    }
}

impl<T: 'static> Clone for Entity<T> {
    fn clone(&self) -> Self {
        // Increment ref count via thread-local store
        with_entity_store(|store| {
            store.increment_ref(self.id);
        });
        Self {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Drop for Entity<T> {
    fn drop(&mut self) {
        // Decrement ref count via thread-local store
        // Note: This may be called outside render context, so we use try_with
        context::try_with_entity_store(|store| {
            store.decrement_ref(self.id);
        });
    }
}

// Entity is Send + Sync if T is Send + Sync
// The actual data lives in EntityStore which handles synchronization
unsafe impl<T: Send> Send for Entity<T> {}
unsafe impl<T: Sync> Sync for Entity<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_equality() {
        let id1 = EntityId::new(0, 1);
        let id2 = EntityId::new(0, 1);
        let id3 = EntityId::new(0, 2);
        let id4 = EntityId::new(1, 1);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3); // Different generation
        assert_ne!(id1, id4); // Different index
    }

    #[test]
    fn test_entity_id_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(EntityId::new(0, 1));
        set.insert(EntityId::new(0, 2));
        set.insert(EntityId::new(1, 1));

        assert_eq!(set.len(), 3);
        assert!(set.contains(&EntityId::new(0, 1)));
    }
}
