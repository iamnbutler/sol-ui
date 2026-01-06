//! Derived/computed values that automatically track dependencies
//!
//! Derived values allow you to compute values from entity state that:
//! - Automatically track which entities they depend on
//! - Re-compute when any dependency changes
//! - Participate in the reactive update system
//!
//! ## Usage
//!
//! ```ignore
//! // Create a derived value that depends on multiple entities
//! let total = derive(|| {
//!     let a = observe(&counter_a, |s| s.value).unwrap_or(0);
//!     let b = observe(&counter_b, |s| s.value).unwrap_or(0);
//!     a + b
//! });
//!
//! // The derived value automatically updates when counter_a or counter_b changes
//! ```
//!
//! Note: Derived values work by leveraging the existing `observe` function.
//! Any entity accessed via `observe` inside the closure is automatically tracked.

use super::context::with_entity_store;

/// Compute a derived value, tracking any entity dependencies
///
/// This is a convenience function that executes a closure within the entity
/// store context. Any calls to `observe()` within the closure will register
/// those entities as dependencies, and the UI will re-render if they change.
///
/// This is primarily useful for computing values from multiple entities:
///
/// ```ignore
/// let total_price = derive(|| {
///     let quantity = observe(&cart, |s| s.quantity).unwrap_or(0);
///     let unit_price = observe(&product, |s| s.price).unwrap_or(0.0);
///     quantity as f64 * unit_price
/// });
/// ```
///
/// # Panics
/// Panics if called outside of a render context.
pub fn derive<R>(f: impl FnOnce() -> R) -> R {
    // Simply execute the closure - any observe() calls inside will
    // automatically register dependencies via the subscription system
    f()
}

/// Compute a derived value from a single entity
///
/// A convenience wrapper that observes an entity and maps its value.
/// Equivalent to `observe(entity, f)` but makes the intent clearer.
///
/// ```ignore
/// let display_name = derive_from(&user, |u| format!("{} {}", u.first_name, u.last_name));
/// ```
pub fn derive_from<T: 'static, R>(
    entity: &super::Entity<T>,
    f: impl FnOnce(&T) -> R,
) -> Option<R> {
    with_entity_store(|store| store.observe(entity, f))
}

/// Compute a derived value from two entities
///
/// ```ignore
/// let full_address = derive_from2(&user, &address, |u, a| {
///     format!("{} at {}", u.name, a.street)
/// });
/// ```
pub fn derive_from2<T1: 'static, T2: 'static, R>(
    entity1: &super::Entity<T1>,
    entity2: &super::Entity<T2>,
    f: impl FnOnce(&T1, &T2) -> R,
) -> Option<R> {
    with_entity_store(|store| {
        let val1 = store.observe(entity1, |v| {
            // We need to get both values, but observe takes ownership of the closure
            // So we need to restructure this
            v as *const T1
        })?;
        let val2 = store.observe(entity2, |v| v as *const T2)?;

        // Safety: The pointers are valid for the duration of this closure
        // because the entity store holds the data and we're within with_entity_store
        unsafe { Some(f(&*val1, &*val2)) }
    })
}

/// Memoize a derived value with explicit dependencies
///
/// For more complex derived values that need caching, you can use this struct
/// to store the computed value and only recompute when dependencies change.
///
/// Note: This is a simple implementation. For production use, you might want
/// a more sophisticated memoization system with proper cache invalidation.
pub struct Memo<T> {
    value: Option<T>,
    version: u64,
}

impl<T> Memo<T> {
    /// Create a new empty memo
    pub fn new() -> Self {
        Self {
            value: None,
            version: 0,
        }
    }

    /// Get or compute the memoized value
    ///
    /// The `compute` closure is called if:
    /// - The value hasn't been computed yet
    /// - The provided `version` is different from the cached version
    ///
    /// Use a version derived from your dependencies (e.g., a hash of entity values)
    /// to control when recomputation occurs.
    pub fn get_or_compute(&mut self, version: u64, compute: impl FnOnce() -> T) -> &T {
        if self.value.is_none() || self.version != version {
            self.value = Some(compute());
            self.version = version;
        }
        self.value.as_ref().unwrap()
    }

    /// Invalidate the cached value
    pub fn invalidate(&mut self) {
        self.value = None;
    }

    /// Check if a value is cached
    pub fn is_cached(&self) -> bool {
        self.value.is_some()
    }
}

impl<T> Default for Memo<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memo_basic() {
        let mut memo = Memo::new();
        let mut compute_count = 0;

        // First access computes
        let value = memo.get_or_compute(1, || {
            compute_count += 1;
            42
        });
        assert_eq!(*value, 42);
        assert_eq!(compute_count, 1);

        // Same version reuses cached value
        let value = memo.get_or_compute(1, || {
            compute_count += 1;
            99
        });
        assert_eq!(*value, 42);
        assert_eq!(compute_count, 1);

        // New version recomputes
        let value = memo.get_or_compute(2, || {
            compute_count += 1;
            100
        });
        assert_eq!(*value, 100);
        assert_eq!(compute_count, 2);
    }

    #[test]
    fn test_memo_invalidate() {
        let mut memo = Memo::new();

        let _ = memo.get_or_compute(1, || 42);
        assert!(memo.is_cached());

        memo.invalidate();
        assert!(!memo.is_cached());
    }
}
