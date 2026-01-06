//! Thread-local access to the entity store
//!
//! This module provides thread-local access to the EntityStore during rendering.
//! The store is set at the beginning of a render frame and cleared at the end.

use super::{Entity, EntityStore};
use std::cell::RefCell;

thread_local! {
    /// Thread-local pointer to the current entity store
    ///
    /// This is set during LayerManager::render() and cleared afterward.
    /// Using a raw pointer because the store's lifetime is managed by App,
    /// and we only access it during the render phase.
    static ENTITY_STORE: RefCell<Option<*mut EntityStore>> = const { RefCell::new(None) };
}

/// Set the current entity store for this thread
///
/// # Safety
/// The caller must ensure the store remains valid for the duration it's set.
/// Call `clear_entity_store()` before the store is dropped.
pub fn set_entity_store(store: &mut EntityStore) {
    ENTITY_STORE.with(|cell| {
        *cell.borrow_mut() = Some(store as *mut EntityStore);
    });
}

/// Clear the current entity store
pub fn clear_entity_store() {
    ENTITY_STORE.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Execute a closure with access to the current entity store
///
/// # Panics
/// Panics if called outside of a render context (when no store is set).
pub fn with_entity_store<R>(f: impl FnOnce(&mut EntityStore) -> R) -> R {
    ENTITY_STORE.with(|cell| {
        let ptr = cell
            .borrow()
            .expect("with_entity_store called outside render context");
        // Safety: We ensure the store is valid while the pointer is set
        let store = unsafe { &mut *ptr };
        f(store)
    })
}

/// Try to execute a closure with access to the current entity store
///
/// Returns None if no store is currently set (e.g., outside render context).
/// This is useful for Drop implementations that may be called at any time.
pub fn try_with_entity_store<R>(f: impl FnOnce(&mut EntityStore) -> R) -> Option<R> {
    ENTITY_STORE.with(|cell| {
        let ptr = *cell.borrow();
        ptr.map(|p| {
            // Safety: We ensure the store is valid while the pointer is set
            let store = unsafe { &mut *p };
            f(store)
        })
    })
}

/// Check if an entity store is currently set
pub fn has_entity_store() -> bool {
    ENTITY_STORE.with(|cell| cell.borrow().is_some())
}

// ============================================================================
// Convenience functions for entity operations
// ============================================================================

/// Create a new entity with the given initial state
///
/// # Panics
/// Panics if called outside of a render context.
///
/// # Example
/// ```ignore
/// let scroll = new_entity(ScrollState::default());
/// ```
pub fn new_entity<T: 'static>(value: T) -> Entity<T> {
    with_entity_store(|store| store.create(value))
}

/// Read entity state immutably
///
/// Returns None if the entity is stale or doesn't exist.
///
/// # Panics
/// Panics if called outside of a render context.
///
/// # Example
/// ```ignore
/// let offset = read_entity(&scroll, |s| s.offset);
/// ```
pub fn read_entity<T: 'static, R>(entity: &Entity<T>, f: impl FnOnce(&T) -> R) -> Option<R> {
    with_entity_store(|store| store.read(entity, f))
}

/// Update entity state mutably
///
/// Returns None if the entity is stale or doesn't exist.
///
/// This automatically marks the entity as dirty, which will trigger a re-render
/// if the entity is being observed via `observe()`.
///
/// # Panics
/// Panics if called outside of a render context.
///
/// # Example
/// ```ignore
/// update_entity(&scroll, |s| s.offset += delta);
/// ```
pub fn update_entity<T: 'static, R>(
    entity: &Entity<T>,
    f: impl FnOnce(&mut T) -> R,
) -> Option<R> {
    with_entity_store(|store| store.update(entity, f))
}

/// Observe entity state (read with automatic re-render on change)
///
/// Like `read_entity`, but also registers interest in this entity's state.
/// If the entity is later mutated via `update_entity`, the system will
/// automatically request a re-render.
///
/// Use `observe` when you want the UI to react to state changes.
/// Use `read_entity` when you just need the current value without reactivity.
///
/// Returns None if the entity is stale or doesn't exist.
///
/// # Panics
/// Panics if called outside of a render context.
///
/// # Example
/// ```ignore
/// // The UI will automatically re-render when counter.value changes
/// let count = observe(&counter, |s| s.value).unwrap_or(0);
/// ```
pub fn observe<T: 'static, R>(entity: &Entity<T>, f: impl FnOnce(&T) -> R) -> Option<R> {
    with_entity_store(|store| store.observe(entity, f))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_and_clear() {
        let mut store = EntityStore::new();

        assert!(!has_entity_store());

        set_entity_store(&mut store);
        assert!(has_entity_store());

        clear_entity_store();
        assert!(!has_entity_store());
    }

    #[test]
    fn test_with_entity_store() {
        let mut store = EntityStore::new();
        set_entity_store(&mut store);

        let result = with_entity_store(|s| {
            s.len() // Just access something
        });

        assert_eq!(result, 0);

        clear_entity_store();
    }

    #[test]
    fn test_try_with_no_store() {
        let result = try_with_entity_store(|_| 42);
        assert!(result.is_none());
    }

    #[test]
    fn test_try_with_store() {
        let mut store = EntityStore::new();
        set_entity_store(&mut store);

        let result = try_with_entity_store(|_| 42);
        assert_eq!(result, Some(42));

        clear_entity_store();
    }

    #[test]
    #[should_panic(expected = "with_entity_store called outside render context")]
    fn test_with_entity_store_panics_outside_context() {
        // Should panic when no store is set
        with_entity_store(|_| {});
    }
}
