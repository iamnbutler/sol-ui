//! Computed/derived values that automatically update when source entities change
//!
//! This module provides `Computed<T, R>` - a derived value that caches
//! the result of a computation on an entity and automatically updates
//! when the source entity changes.

use super::observe::SubscriptionId;
use super::{Entity, EntityId};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

/// A computed value derived from an entity
///
/// `Computed<T, R>` wraps an entity and a mapping function, caching the result.
/// When the source entity changes, the cached value is invalidated and
/// will be recomputed on next access.
///
/// # Example
/// ```ignore
/// let counter = new_entity(CounterState { count: 0 });
/// let doubled = Computed::new(&counter, |state| state.count * 2);
///
/// // Read the computed value
/// let value = doubled.get(); // Returns 0
///
/// // Update the source
/// update_entity(&counter, |s| s.count = 5);
///
/// // Computed value is now stale and will recompute on next access
/// let value = doubled.get(); // Returns 10
/// ```
pub struct Computed<T: 'static, R: Clone + 'static> {
    /// The source entity
    source: Entity<T>,
    /// The mapping function
    mapper: Rc<dyn Fn(&T) -> R>,
    /// Cached value and validity flag
    cache: Rc<RefCell<ComputedCache<R>>>,
    /// Subscription to source entity changes
    subscription_id: Option<SubscriptionId>,
    /// Phantom for R
    _marker: PhantomData<R>,
}

/// Internal cache state for computed values
struct ComputedCache<R> {
    /// The cached value (if computed)
    value: Option<R>,
    /// Whether the cache is valid
    valid: bool,
}

impl<R> ComputedCache<R> {
    fn new() -> Self {
        Self {
            value: None,
            valid: false,
        }
    }

    fn invalidate(&mut self) {
        self.valid = false;
    }

    fn set(&mut self, value: R) {
        self.value = Some(value);
        self.valid = true;
    }

    fn get(&self) -> Option<&R> {
        if self.valid {
            self.value.as_ref()
        } else {
            None
        }
    }
}

impl<T: 'static, R: Clone + 'static> Computed<T, R> {
    /// Create a new computed value from an entity and a mapping function
    ///
    /// The mapping function will be called to compute the derived value,
    /// and the result will be cached until the source entity changes.
    ///
    /// Note: This must be called within a render context to set up the subscription.
    pub fn new<F>(source: &Entity<T>, mapper: F) -> Self
    where
        F: Fn(&T) -> R + 'static,
    {
        let cache = Rc::new(RefCell::new(ComputedCache::new()));

        // Set up subscription to invalidate cache when source changes
        let cache_for_sub = cache.clone();
        let subscription_id = super::context::try_with_entity_store(|store| {
            store.subscribe(source, Box::new(move || {
                cache_for_sub.borrow_mut().invalidate();
            }))
        });

        Self {
            source: source.clone(),
            mapper: Rc::new(mapper),
            cache,
            subscription_id,
            _marker: PhantomData,
        }
    }

    /// Get the computed value
    ///
    /// If the cache is valid, returns the cached value.
    /// Otherwise, recomputes from the source entity.
    ///
    /// Returns None if the source entity is stale/invalid.
    pub fn get(&self) -> Option<R> {
        // Check if cache is valid
        if let Some(value) = self.cache.borrow().get() {
            return Some(value.clone());
        }

        // Recompute from source
        let value = super::context::try_with_entity_store(|store| {
            store.read(&self.source, |state| (self.mapper)(state))
        })??;

        // Update cache
        self.cache.borrow_mut().set(value.clone());
        Some(value)
    }

    /// Force invalidation of the cached value
    ///
    /// The next call to `get()` will recompute the value.
    pub fn invalidate(&self) {
        self.cache.borrow_mut().invalidate();
    }

    /// Check if the cache is currently valid
    pub fn is_valid(&self) -> bool {
        self.cache.borrow().valid
    }

    /// Get the source entity's ID
    pub fn source_id(&self) -> EntityId {
        self.source.id()
    }
}

impl<T: 'static, R: Clone + 'static> Clone for Computed<T, R> {
    fn clone(&self) -> Self {
        // Clone shares the same cache but sets up a new subscription
        let cache = self.cache.clone();
        let cache_for_sub = cache.clone();

        let subscription_id = super::context::try_with_entity_store(|store| {
            store.subscribe(&self.source, Box::new(move || {
                cache_for_sub.borrow_mut().invalidate();
            }))
        });

        Self {
            source: self.source.clone(),
            mapper: self.mapper.clone(),
            cache,
            subscription_id,
            _marker: PhantomData,
        }
    }
}

impl<T: 'static, R: Clone + 'static> Drop for Computed<T, R> {
    fn drop(&mut self) {
        // Unsubscribe when dropped
        if let Some(id) = self.subscription_id {
            super::context::try_with_entity_store(|store| {
                store.unsubscribe(id);
            });
        }
    }
}

/// Create a computed value from an entity
///
/// This is a convenience function for creating `Computed` values.
///
/// # Example
/// ```ignore
/// let counter = new_entity(CounterState { count: 0 });
/// let doubled = computed(&counter, |s| s.count * 2);
/// ```
pub fn computed<T: 'static, R: Clone + 'static, F>(source: &Entity<T>, mapper: F) -> Computed<T, R>
where
    F: Fn(&T) -> R + 'static,
{
    Computed::new(source, mapper)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityStore, set_entity_store, clear_entity_store};

    #[derive(Debug)]
    struct TestState {
        value: i32,
    }

    #[test]
    fn test_computed_basic() {
        let mut store = EntityStore::new();
        set_entity_store(&mut store);

        let entity = store.create(TestState { value: 5 });
        let doubled = Computed::new(&entity, |s| s.value * 2);

        // Initial value
        assert_eq!(doubled.get(), Some(10));
        assert!(doubled.is_valid());

        // Update source
        store.update(&entity, |s| s.value = 10);

        // Cache should be invalidated (but notification happens on flush)
        store.flush_notifications();
        assert!(!doubled.is_valid());

        // Get should recompute
        assert_eq!(doubled.get(), Some(20));
        assert!(doubled.is_valid());

        clear_entity_store();
    }

    #[test]
    fn test_computed_invalidate() {
        let mut store = EntityStore::new();
        set_entity_store(&mut store);

        let entity = store.create(TestState { value: 5 });
        let doubled = Computed::new(&entity, |s| s.value * 2);

        // Get initial value
        assert_eq!(doubled.get(), Some(10));
        assert!(doubled.is_valid());

        // Manual invalidate
        doubled.invalidate();
        assert!(!doubled.is_valid());

        // Should recompute (same value since source unchanged)
        assert_eq!(doubled.get(), Some(10));

        clear_entity_store();
    }
}
